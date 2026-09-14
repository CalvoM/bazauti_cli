use bazauti::parser::errors::RenderingError;
use bazauti::parser::utils::CompressionCode;
use bazauti::parser::wav::{AudioData, WaveParserContext};
use plotters::prelude::*;

const WIDTH: u32 = 1600;
const HEIGHT: u32 = 700;

#[derive(Clone, Debug)]
pub struct WaveformRenderer {
    sample_rate: u32,
    number_of_channels: u16,
    compression_code: CompressionCode,
    data: Option<AudioData>,
}

impl WaveformRenderer {
    pub fn new(wave_parser_ctx: WaveParserContext) -> WaveformRenderer {
        Self {
            sample_rate: wave_parser_ctx.sample_rate,
            number_of_channels: wave_parser_ctx.channels,
            compression_code: wave_parser_ctx.compression_code,
            data: wave_parser_ctx.samples,
        }
    }
    pub fn render(self, start_time: f64, duration: f64) {
        match self.data.as_ref().unwrap() {
            AudioData::U8(samples) => self
                .clone()
                .plot_u8(&samples, start_time, duration)
                .unwrap(),
            AudioData::I16(samples) => self
                .clone()
                .plot_i16(&samples, start_time, duration)
                .unwrap(),
        }
    }
    fn plot_i16(
        &self,
        samples: &[i16],
        start_time: f64,
        duration: f64,
    ) -> Result<(), RenderingError> {
        let sample_rate = self.sample_rate as f64;
        let channels = self.number_of_channels;

        let start_sample = (start_time * sample_rate) as usize;
        let sample_count = (duration * sample_rate) as usize;

        let start_sample = start_sample.min(samples.len());

        let samples = match channels {
            1 => {
                let end = (start_sample + sample_count).min(samples.len());
                &samples[start_sample..end]
            }
            2 => {
                let start = (start_sample * 2).min(samples.len());
                let end = ((start_sample + sample_count) * 2).min(samples.len());
                &samples[start..end]
            }
            _ => {
                return Err(RenderingError::ChartSetupError(format!(
                    "Unsupported channel count: {channels}"
                )));
            }
        };

        let root = BitMapBackend::new("waveform.png", (WIDTH, HEIGHT)).into_drawing_area();

        let _ = root
            .fill(&WHITE)
            .map_err(|e| RenderingError::ChartSetupError(e.to_string()));

        match channels {
            1 => {
                self.clone().draw_i16_channel(
                    &root,
                    samples,
                    sample_rate,
                    start_time,
                    duration,
                    "Mono",
                )?;
            }

            2 => {
                let areas = root.split_evenly((2, 1));

                self.clone().draw_i16_channel(
                    &areas[0],
                    samples
                        .iter()
                        .step_by(2)
                        .copied()
                        .collect::<Vec<_>>()
                        .as_slice(),
                    sample_rate,
                    start_time,
                    duration,
                    "Left",
                )?;

                self.clone().draw_i16_channel(
                    &areas[1],
                    samples
                        .iter()
                        .skip(1)
                        .step_by(2)
                        .copied()
                        .collect::<Vec<_>>()
                        .as_slice(),
                    sample_rate,
                    start_time,
                    duration,
                    "Right",
                )?;
            }

            _ => unreachable!(),
        }

        let _ = root
            .present()
            .map_err(|e| RenderingError::ChartDrawError(e.to_string()));

        Ok(())
    }

    fn draw_i16_channel<DB: DrawingBackend>(
        self,
        area: &DrawingArea<DB, plotters::coord::Shift>,
        samples: &[i16],
        sample_rate: f64,
        start_time: f64,
        duration: f64,
        channel_name: &str,
    ) -> Result<(), RenderingError>
    where
        DB::ErrorType: 'static,
    {
        let mut chart = ChartBuilder::on(area)
            .caption(
                format!("{} Waveform - {channel_name}", self.compression_code),
                ("sans-serif", 20),
            )
            .margin(15)
            .x_label_area_size(40)
            .y_label_area_size(60)
            .build_cartesian_2d(
                start_time..start_time + duration,
                i32::from(i16::MIN)..i32::from(i16::MAX),
            )
            .map_err(|e| RenderingError::ChartSetupError(e.to_string()))?;

        chart
            .configure_mesh()
            .x_desc("Time (s)")
            .y_desc("Amplitude")
            .draw()
            .map_err(|e| RenderingError::ChartDrawError(e.to_string()))?;

        chart
            .draw_series(LineSeries::new(
                samples.iter().enumerate().map(|(i, &sample)| {
                    let time = start_time + i as f64 / sample_rate;

                    (time, i32::from(sample))
                }),
                &BLUE,
            ))
            .map_err(|e| RenderingError::ChartDrawError(e.to_string()))?;

        chart
            .draw_series(LineSeries::new(
                [(start_time, 0), (start_time + duration, 0)],
                &BLACK.mix(0.3),
            ))
            .map_err(|e| RenderingError::ChartDrawError(e.to_string()))?;

        Ok(())
    }
    fn plot_u8(self, samples: &[u8], start_time: f64, duration: f64) -> Result<(), RenderingError> {
        let sample_rate = self.sample_rate as f64;
        let channels = self.number_of_channels;

        let start_frame = (start_time * sample_rate) as usize;
        let frame_count = (duration * sample_rate) as usize;

        let start = match channels {
            1 => start_frame,
            2 => start_frame * 2,
            _ => {
                return Err(RenderingError::ChartSetupError(format!(
                    "Unsupported channel count: {channels}"
                )));
            }
        };

        let end = match channels {
            1 => (start_frame + frame_count).min(samples.len()),
            2 => ((start_frame + frame_count) * 2).min(samples.len()),
            _ => unreachable!(),
        };

        if start >= samples.len() || start >= end {
            return Err(RenderingError::ChartSetupError(
                "Requested time range contains no samples".into(),
            ));
        }

        let samples = &samples[start..end];

        let root = BitMapBackend::new("waveform.png", (1600, 700)).into_drawing_area();

        let _ = root
            .fill(&WHITE)
            .map_err(|e| RenderingError::ChartDrawError(e.to_string()));

        match channels {
            1 => {
                self.draw_u8_channel(
                    &root,
                    samples.iter().copied(),
                    sample_rate,
                    start_time,
                    duration,
                    "Mono",
                )?;
            }

            2 => {
                let areas = root.split_evenly((2, 1));

                self.clone().draw_u8_channel(
                    &areas[0],
                    samples.iter().step_by(2).copied(),
                    sample_rate,
                    start_time,
                    duration,
                    "Left",
                )?;

                self.draw_u8_channel(
                    &areas[1],
                    samples.iter().skip(1).step_by(2).copied(),
                    sample_rate,
                    start_time,
                    duration,
                    "Right",
                )?;
            }

            _ => unreachable!(),
        }

        let _ = root
            .present()
            .map_err(|e| RenderingError::ChartDrawError(e.to_string()));

        Ok(())
    }
    fn draw_u8_channel<DB, I>(
        self,
        area: &DrawingArea<DB, plotters::coord::Shift>,
        samples: I,
        sample_rate: f64,
        start_time: f64,
        duration: f64,
        channel_name: &str,
    ) -> Result<(), RenderingError>
    where
        DB: DrawingBackend,
        DB::ErrorType: 'static,
        I: Iterator<Item = u8>,
    {
        let mut chart = ChartBuilder::on(area)
            .caption(
                format!("{} Waveform - {channel_name}", self.compression_code),
                ("sans-serif", 20),
            )
            .margin(15)
            .x_label_area_size(40)
            .y_label_area_size(60)
            .build_cartesian_2d(start_time..start_time + duration, 0u32..255u32)
            .map_err(|e| RenderingError::ChartSetupError(e.to_string()))?;

        chart
            .configure_mesh()
            .x_desc("Time (s)")
            .y_desc("Amplitude")
            .draw()
            .map_err(|e| RenderingError::ChartDrawError(e.to_string()))?;

        chart
            .draw_series(LineSeries::new(
                samples.enumerate().map(|(i, sample)| {
                    let time = start_time + i as f64 / sample_rate;

                    (time, u32::from(sample))
                }),
                &BLUE,
            ))
            .map_err(|e| RenderingError::ChartSetupError(e.to_string()))?;

        // 8-bit PCM silence is centered at 128.
        chart
            .draw_series(LineSeries::new(
                [(start_time, 128), (start_time + duration, 128)],
                &BLACK.mix(0.3),
            ))
            .map_err(|e| RenderingError::ChartSetupError(e.to_string()))?;

        Ok(())
    }
}
