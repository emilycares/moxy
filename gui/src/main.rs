use eframe::{
    egui::{self, CentralPanel, FontData, FontDefinitions, ScrollArea},
    epaint::FontFamily,
    epi::App,
    run_native, NativeOptions,
};

#[derive(Debug)]
struct Gui {
    requests: Vec<Request>,
}

impl Gui {
    fn new() -> Self {
        let items = (0..20).map(|a| Request {
            url: format!("https://asd.com/{}", a),
            content: format!("content: {}", a),
        });
        Self {
            requests: Vec::from_iter(items),
        }
    }

    fn configure_fonts(&self, ctx: &eframe::egui::Context) {
        //let mut fonts = FontDefinitions::default();
        //fonts.font_data.insert(
            //"SauceCodePro".to_owned(),
            //FontData::from_static(include_bytes!("../assets/SauceCodeProMediumNF.ttf")),
        //);

        //// Put my font first (highest priority):
        //fonts
            //.families
            //.get_mut(&FontFamily::Proportional)
            //.unwrap()
            //.insert(0, "SauceCodePro".to_owned());

        //// Put my font as last fallback for monospace:
        //fonts
            //.families
            //.get_mut(&FontFamily::Monospace)
            //.unwrap()
            //.push("SauceCodePro".to_owned());

        //ctx.set_fonts(fonts);
    }
}

#[derive(Debug)]
struct Request {
    url: String,
    content: String,
}

impl App for Gui {
    fn setup(
        &mut self,
        ctx: &eframe::egui::Context,
        _frame: &eframe::epi::Frame,
        _storage: Option<&dyn eframe::epi::Storage>,
    ) {
        self.configure_fonts(ctx);
    }

    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &eframe::epi::Frame) {
        CentralPanel::default().show(ctx, |ui| {
            egui::SidePanel::left("requests")
                .resizable(true)
                .show_inside(ui, |ui| {
                    ScrollArea::vertical().id_source(1).show(ui, |ui| {
                        for a in &self.requests {
                            ui.label(&a.url);
                            ui.label(&a.content);
                        }
                    });
                });
            egui::CentralPanel::default()
                .show_inside(ui, |ui| {
                    ScrollArea::vertical().id_source(1).show(ui, |ui| {
                        for a in &self.requests {
                            ui.label(&a.url);
                            ui.label(&a.content);
                        }
                    });
                });
        });
    }

    fn name(&self) -> &str {
        "moxy-gui"
    }
    // add code here
}

fn main() {
    let app = Gui::new();
    let window_option = NativeOptions::default();
    run_native(Box::new(app), window_option);
}
