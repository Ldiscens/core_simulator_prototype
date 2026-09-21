use std::ops::RangeInclusive;
use eframe::egui;
use crate::system::{System, reset_system, next_dt};
use egui_plot::{Plot, Legend, PlotPoints, Points, AxisHints, GridMark};

struct Progress {
    time: Vec<f64>,
    power: Vec<f64>,
}

impl Default for Progress {
    fn default() -> Self {
        let mut time = Vec::with_capacity(1000);
        time.push(0.0);
        let mut power = Vec::with_capacity(1000);
        power.push(0.0);
        Self {
            time: time,
            power: power,
        }
    }
}

struct UiState {
    system: System,
    is_running: bool,
    time_multiplier: f64,
    progress: Progress,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            system: System::default(),
            is_running: false,
            time_multiplier: 1.0,
            progress: Progress::default(),
        }
    }
}

impl UiState {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }
}

fn do_real_time_25ms(state: &mut UiState) {
    let steps_per_25ms = (2500.0*state.time_multiplier) as isize;
    let y_a = state.system.neutron_pop;
    
    for _i in 0..steps_per_25ms {
        next_dt(&mut state.system);
    }

    let y_b = state.system.neutron_pop;
    if y_b < y_a {
        state.system.doubling_time_estimator = 0.0;
    }
    else {
        let a = (y_b - y_a)/(state.system.dt*(steps_per_25ms as f64));
        state.system.doubling_time_estimator = y_a / a;
    }
    state.system.power_estimator = (state.system.neutron_pop/2.5)*200e6*1.6e-19*state.system.generation_time;

    if state.system.is_scraming {
        if state.system.rods > -8750e-5 {
            state.system.rods -= 1250.0e-5;
        } else if state.system.rods > -10000e-5 {
            state.system.rods = -10000e-5;
            state.system.is_scraming = false;
        }
    }

    state.progress.time.push(state.system.current_time);
    state.progress.power.push(state.system.power_estimator);
    if state.progress.time.len() == 1000 {
        state.progress.time.drain(..800);
        state.progress.power.drain(..800);
    }
}

impl eframe::App for UiState {

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if self.is_running {
            do_real_time_25ms(self);
            ui.request_repaint();
        }
        let precursors_total_pop: f64 = self.system.precursors_pop.iter().sum();

        egui::CentralPanel::default().show(ui, |ui| 
            {
                ui.heading("Core simulator");
                ui.horizontal(|ui| {
                        ui.label(format!("Time [s] (x{:.2}): ", self.time_multiplier));
                        ui.label(format!("{:.2}", self.system.current_time));
                    }
                );
                ui.horizontal(|ui| 
                    {
                        if ui.add(egui::Button::new("<<"))
                            .clicked() {
                                self.time_multiplier *= 0.5;
                            }
                        if ui.add(egui::Button::new("▶⏸"))
                            .clicked() {
                                self.is_running = !self.is_running;
                            }
                        if ui.add(egui::Button::new(">>"))
                            .clicked() {
                                self.time_multiplier *= 2.0;
                            }
                        if ui.add(egui::Button::new("RAZ"))
                            .clicked() {
                                reset_system(&mut self.system);
                            }
                    }
                );
                ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                                ui.label("Power estimate [W]");
                                ui.label("Neutron population");
                                ui.label("Precursor population");
                                ui.label("Reactivity");
                                ui.label("β eff");
                                ui.label("Doubling time [s]");
                                ui.label("External source activity [Bq]");
                                ui.label("Boric acid");
                                ui.label("Depletion");
                                ui.label("Rods");
                            }
                        );
                        ui.vertical(|ui| {
                                ui.label(format!("{:.2e}", self.system.power_estimator));
                                ui.label(format!("{:.3e}", self.system.neutron_pop));
                                ui.label(format!("{:.3e}", precursors_total_pop));
                                ui.label(format!("{:.5}", self.system.reactivity));
                                ui.label(format!("{:.5}", self.system.effective_beta));
                                ui.label(format!("{:.1}", self.system.doubling_time_estimator));
                                ui.label(format!("{:.3e}", if self.system.external_source_is_in {self.system.external_source_activity} else {0.0}));
                                ui.label(format!("{:.5}", self.system.boric_acid));
                                ui.label(format!("{:.5}", self.system.depletion));
                                ui.label(format!("{:.5}", self.system.rods));
                            }
                        );
                        ui.vertical(|ui| {
                                if ui.add(egui::Button::new("↑↑↑↑")).clicked() {self.system.rods += 1000e-5;}
                                if ui.add(egui::Button::new("↑↑↑")).clicked() {self.system.rods += 100e-5;}
                                if ui.add(egui::Button::new("↑↑")).clicked() {self.system.rods += 10e-5;}
                                if ui.add(egui::Button::new("↑")).clicked() {self.system.rods += 2e-5;}
                                if ui.add(egui::Button::new("↓")).clicked() {self.system.rods -= 2e-5;}
                                if ui.add(egui::Button::new("↓↓")).clicked() {self.system.rods -= 10e-5;}
                                if ui.add(egui::Button::new("↓↓↓")).clicked() {self.system.rods -= 100e-5;}
                                if ui.add(egui::Button::new("↓↓↓↓")).clicked() {self.system.rods -= 1000e-5;}
                                if ui.add(egui::Button::new("SCRAM")).clicked() {self.system.is_scraming =true;}
                                if ui.add(egui::Button::new("Add / Remove source")).clicked() {self.system.external_source_is_in = !self.system.external_source_is_in;}
                            }
                        );
                    }
                );
                Plot::new("power_plot")
                    .show_axes(true)
                    .custom_x_axes(vec![
                        AxisHints::new_x()
                            .label("Time [s]")
                        ]
                    )
                    .custom_y_axes(vec![
                        AxisHints::new_y()
                            .label("Power [s]")
                            .formatter(|mark: GridMark, _range: &RangeInclusive<f64>| {
                                    format!("{:.2e}", mark.value)
                                }
                            )
                        ]
                    )
                    .show(ui, |plot_ui| {
                            plot_ui.points(
                                Points::new("markers", 
                                    (0..self.progress.time.len())
                                    .map(|i| {let x = self.progress.time[i]; let y = self.progress.power[i]; [x, y]})
                                    .collect::<Vec<_>>()
                                )
                            )
                        }
                    )
                    .response
            }
        );
    }

    // fn plot(&mut self, ui: &mut egui::Ui) {
    //     let mut plot = Plot::new()
    //         .legend(Legend::default())
    //         .show_axes(true)
    //         .show_grid(true)

    //     let power_points = Points::new("markers", 
    //         (0..self.progress.time.len())
    //         .map(|i| {let x = self.progress.times[i]; let y = self.progress.power[i]; [x, y]}).collect()
    //     );
    // }
}



pub fn run_ui() {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Core simulator",
        native_options,
        Box::new(|cc| 
            Ok(Box::new(UiState::new(cc)))
        ),
    );
}