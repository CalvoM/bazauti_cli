use bazauti::parser::wav::{AudioData, WAVParser};
use comfy_table::{presets::UTF8_FULL, *};

pub fn summary(parser: &WAVParser) {
    let metadata = parser.clone().metadata();
    let fmt = &metadata.fmt_metadata;
    let context = parser.clone().context();

    let sample_count = match &context.samples {
        Some(AudioData::U8(samples)) => samples.len(),
        Some(AudioData::I16(samples)) => samples.len(),
        None => 0,
    };
    let frame_count = if fmt.number_of_channels > 0 {
        sample_count / fmt.number_of_channels as usize
    } else {
        0
    };
    let duration_secs = if fmt.sample_rate_per_second > 0 {
        frame_count as f64 / fmt.sample_rate_per_second as f64
    } else {
        0.0
    };
    let bit_rate_kbps = fmt.avg_bytes_per_second as f64 * 8.0 / 1000.0;

    let mut table = Table::new();
    table
        .load_style(UTF8_FULL.with_rounded_corners())
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_width(40)
        .set_header(vec!["Attribute", "Value"])
        .add_row(vec![
            Cell::new("Compression Type").set_alignment(CellAlignment::Center),
            Cell::new(fmt.compression_code).set_alignment(CellAlignment::Center),
        ])
        .add_row(vec![
            Cell::new("Sample rate").set_alignment(CellAlignment::Center),
            Cell::new(fmt.sample_rate_per_second).set_alignment(CellAlignment::Center),
        ])
        .add_row(vec![
            Cell::new("Number of Channels").set_alignment(CellAlignment::Center),
            Cell::new(fmt.number_of_channels).set_alignment(CellAlignment::Center),
        ])
        .add_row(vec![
            Cell::new("Bits per sample").set_alignment(CellAlignment::Center),
            Cell::new(fmt.bits_per_sample).set_alignment(CellAlignment::Center),
        ])
        .add_row(vec![
            Cell::new("Block align").set_alignment(CellAlignment::Center),
            Cell::new(fmt.block_align).set_alignment(CellAlignment::Center),
        ])
        .add_row(vec![
            Cell::new("Average bytes per second").set_alignment(CellAlignment::Center),
            Cell::new(fmt.avg_bytes_per_second).set_alignment(CellAlignment::Center),
        ])
        .add_row(vec![
            Cell::new("Bit rate (kbps)").set_alignment(CellAlignment::Center),
            Cell::new(format!("{bit_rate_kbps:.1}")).set_alignment(CellAlignment::Center),
        ])
        .add_row(vec![
            Cell::new("File size (bytes)").set_alignment(CellAlignment::Center),
            Cell::new(metadata.file_size).set_alignment(CellAlignment::Center),
        ])
        .add_row(vec![
            Cell::new("Duration (s)").set_alignment(CellAlignment::Center),
            Cell::new(format!("{duration_secs:.2}")).set_alignment(CellAlignment::Center),
        ]);
    println!("{table}");
}
