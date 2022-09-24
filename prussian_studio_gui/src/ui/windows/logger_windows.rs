use egui::{Color32, ComboBox, Grid, Slider, Window};
use lib_logger::{parse_pattern, Logger, LoggerType};
use regex::Regex;
use rfd::FileDialog;

use crate::window::{LoggerWindowBuffer, WindowsOpen};

pub fn logger_config_window(
    windows_open: &mut WindowsOpen,
    ctx: &egui::Context,
    logger_window_buffer: &mut LoggerWindowBuffer,
    re: &mut (Regex, Regex),
    loggers: &mut Vec<Logger>,
) {
    Window::new("Configure Logger")
        .open(&mut windows_open.logger_configure)
        .scroll2([true, true])
        .show(ctx, |ui| {
            Grid::new("Logger List")
                .striped(true)
                .num_columns(2)
                .show(ui, |ui| {
                    ui.label("Logger name:");
                    ui.add(egui::TextEdit::singleline(
                        &mut logger_window_buffer.logger_name,
                    ));
                    ui.end_row();
                    ui.label("Type");
                    ComboBox::from_label("")
                        .selected_text(format!("{}", logger_window_buffer.logger_type))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut logger_window_buffer.logger_type,
                                LoggerType::DataBase,
                                "Database",
                            );
                            ui.selectable_value(
                                &mut logger_window_buffer.logger_type,
                                LoggerType::TextFile,
                                "Text File",
                            );
                        });
                    ui.end_row();
                    ui.label("Log rate:");
                    ui.add(
                        Slider::new(&mut logger_window_buffer.log_rate, 1..=900).text("Seconds"),
                    );
                    ui.end_row();
                    ui.label("Channel pattern:");
                    ui.add(
                        egui::TextEdit::singleline(
                            &mut logger_window_buffer.channel_pattern.pattern,
                        )
                        .hint_text("Ex: CH1-CH7, EVAL10-EVAL20"),
                    );
                    ui.end_row();
                    match parse_pattern(&logger_window_buffer.channel_pattern, (&re.0, &re.1)) {
                        Ok(channels) => {
                            ui.colored_label(
                                Color32::DARK_GREEN,
                                format!("{} channels will be logged.", &channels.len()),
                            );
                        }
                        Err(e) => {
                            ui.colored_label(Color32::RED, format!("{}", e));
                        }
                    }
                    ui.end_row();
                    if ui.button("Path").clicked() {
                        if let Some(path) = FileDialog::new().set_directory(".").save_file() {
                            logger_window_buffer.path = path;
                        }
                    }
                    ui.label(format!("{}", &logger_window_buffer.path.to_str().unwrap()));
                    ui.end_row();
                    ui.label("Logger list");
                    ui.end_row();
                    for logger in loggers.iter() {
                        ui.label("Name:");
                        ui.label(format!("{}", &logger.name));
                        ui.end_row();
                        ui.label("Number of channels:");
                        ui.label(format!("{}", &logger.channels.len()));
                        ui.end_row();
                        ui.label("Type:");
                        ui.label(format!("{}", &logger.logger_type));
                        ui.end_row();
                        ui.label("Logging rate:");
                        ui.label(format!("{} seconds", &logger.log_rate));
                        ui.end_row();
                        ui.separator();
                        ui.end_row();
                    }
                    ui.vertical_centered_justified(|ui| {
                        if ui.button("Save").clicked() {
                            let logger = Logger::new(
                                logger_window_buffer.logger_name.clone(),
                                logger_window_buffer.logger_type.clone(),
                                &mut logger_window_buffer.channel_pattern,
                                logger_window_buffer.path.clone(),
                                logger_window_buffer.log_rate,
                                false,
                                (&re.0, &re.1),
                            );

                            if let Ok(logger) = logger {
                                loggers.push(logger);
                            }
                        }
                    });
                });
        });
}
