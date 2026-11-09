//! Fanoratana sur Linux (et tout bureau) : la même interface que sur la tablette.

use std::path::PathBuf;

use fano_ui::{Fanoratana, Plateforme};

struct Bureau {
    image: Option<(String, Vec<u8>)>,
}

impl Plateforme for Bureau {
    fn dossier_donnees(&self) -> PathBuf {
        std::env::var_os("FANORATANA_DONNEES")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                dirs::data_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join("fanoratana")
            })
    }

    fn demander_image(&mut self) {
        let f = rfd::FileDialog::new()
            .add_filter("Images", &["png", "jpg", "jpeg", "svg"])
            .pick_file();
        if let Some(p) = f {
            if let Ok(o) = std::fs::read(&p) {
                let nom = p
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| "image.png".into());
                self.image = Some((nom, o));
            }
        }
    }

    fn demander_document(&mut self) {
        let f = rfd::FileDialog::new()
            .add_filter("PDF, images", &["pdf", "png", "jpg", "jpeg", "svg"])
            .pick_file();
        if let Some(p) = f {
            if let Ok(o) = std::fs::read(&p) {
                let nom = p
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_else(|| "document.pdf".into());
                self.image = Some((nom, o));
            }
        }
    }

    fn image_choisie(&mut self) -> Option<(String, Vec<u8>)> {
        self.image.take()
    }

    fn enregistrer(&mut self, nom: &str, _mime: &str, octets: Vec<u8>) -> Result<Option<String>, String> {
        let ext = nom.rsplit('.').next().unwrap_or("");
        let Some(p) = rfd::FileDialog::new()
            .set_file_name(nom)
            .add_filter(ext, &[ext])
            .set_directory(dirs::document_dir().unwrap_or_default())
            .save_file()
        else {
            return Ok(None);
        };
        std::fs::write(&p, octets).map_err(|e| e.to_string())?;
        Ok(Some(p.display().to_string()))
    }
}

struct Appli(Fanoratana);

impl eframe::App for Appli {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.0.logic(ctx);
    }
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.0.ui(ui);
    }
    fn on_exit(&mut self) {
        self.0.quitter();
    }
}

fn main() -> eframe::Result {
    env_logger::init();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Fanoratana")
            .with_app_id("mg.fanoratana.app")
            .with_inner_size([1200.0, 860.0])
            .with_min_inner_size([420.0, 480.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Fanoratana",
        options,
        Box::new(|cc| {
            Ok(Box::new(Appli(Fanoratana::new(
                &cc.egui_ctx,
                Box::new(Bureau { image: None }),
            ))))
        }),
    )
}
