//! Fanoratana sur Android : la passerelle (fano-android, reprise de mahay) et la même interface
//! que sur Linux.

#[cfg(target_os = "android")]
mod android {
    use std::cell::Cell;
    use std::path::PathBuf;
    use std::rc::Rc;
    use std::sync::mpsc::Receiver;

    use fano_android::{AndroidApp, EguiBridge, Picked, ToolType};
    use fano_ui::{Fanoratana, Plateforme, Pointeur};

    struct Tablette {
        image: Option<Receiver<Option<Picked>>>,
        clavier: Rc<Cell<Option<bool>>>,
        /// Les marges du système, relues au plus toutes les 400 ms (appel JNI).
        marges: ([f32; 4], Option<std::time::Instant>),
    }

    impl Plateforme for Tablette {
        fn dossier_donnees(&self) -> PathBuf {
            fano_android::data_dir().unwrap_or_else(|| PathBuf::from("/data/local/tmp")).join("fanoratana")
        }
        fn demander_image(&mut self) {
            self.image = Some(fano_android::pick_document(&["image/png", "image/jpeg", "image/svg+xml"]));
        }
        fn image_choisie(&mut self) -> Option<(String, Vec<u8>)> {
            let rx = self.image.as_ref()?;
            match rx.try_recv() {
                Ok(Some(p)) => {
                    self.image = None;
                    let octets = std::fs::read(&p.path).ok()?;
                    Some((p.name, octets))
                }
                Ok(None) | Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                    self.image = None;
                    None
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => None,
            }
        }
        fn enregistrer(&mut self, nom: &str, _mime: &str, octets: Vec<u8>) -> Result<Option<String>, String> {
            // Android demande où ranger le fichier (Documents, Drive…) dès la première écriture.
            let chemin = fano_android::save_target(nom);
            std::fs::write(&chemin, octets).map_err(|e| e.to_string())?;
            Ok(Some(nom.to_owned()))
        }
        fn pointeur(&self, toucher: Option<u64>) -> Pointeur {
            match toucher.map(fano_android::tool_type) {
                Some(ToolType::Stylus | ToolType::Eraser) => Pointeur::Stylet,
                Some(ToolType::Mouse) => Pointeur::Souris,
                Some(_) => Pointeur::Doigt,
                None => Pointeur::Souris,
            }
        }
        fn clavier(&mut self, visible: bool) {
            self.clavier.set(Some(visible));
        }
        fn langue(&self) -> String {
            fano_android::locale()
        }
        fn marges(&mut self) -> [f32; 4] {
            if self.marges.1.is_none_or(|t| t.elapsed() >= std::time::Duration::from_millis(400)) {
                self.marges = (fano_android::system_insets().map(|v| v.max(0) as f32), Some(std::time::Instant::now()));
            }
            self.marges.0
        }
        fn installer_maj(&mut self, fichier: &std::path::Path) -> Result<String, String> {
            fano_android::install_apk(fichier)?;
            Ok(if fano_android::locale().starts_with("en") {
                "Android will ask you to confirm the installation.".into()
            } else {
                "Android va demander de confirmer l'installation.".into()
            })
        }
    }

    struct Appli {
        app: Fanoratana,
        pont: EguiBridge,
        clavier: Rc<Cell<Option<bool>>>,
    }

    impl eframe::App for Appli {
        fn raw_input_hook(&mut self, _ctx: &egui::Context, raw: &mut egui::RawInput) {
            self.pont.raw_input_hook(raw);
        }
        fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
            // Copie les exports vers l'endroit choisi par la personne.
            let _ = fano_android::poll();
            self.app.logic(ctx);
        }
        fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
            if let Some(v) = self.clavier.take() {
                self.pont.vouloir_clavier(v);
            }
            let app = &mut self.app;
            self.pont.inset_ui(ui, |ui| app.ui(ui));
            self.pont.after_ui(&ui.ctx().clone());
        }
        fn on_exit(&mut self) {
            self.app.quitter();
        }
    }

    #[unsafe(no_mangle)]
    fn android_main(app: AndroidApp) {
        fano_android::init(&app, "Fanoratana");
        let options = fano_android::native_options!(app);
        let r = eframe::run_native(
            "Fanoratana",
            options,
            Box::new(|cc| {
                let clavier = Rc::new(Cell::new(None));
                let plateforme = Tablette { image: None, clavier: clavier.clone(), marges: ([0.0; 4], None) };
                Ok(Box::new(Appli {
                    app: Fanoratana::new(&cc.egui_ctx, Box::new(plateforme)),
                    pont: EguiBridge::new(&cc.egui_ctx),
                    clavier,
                }))
            }),
        );
        if let Err(e) = r {
            log::error!("Fanoratana : {e}");
            fano_android::show_error(&format!("Fanoratana n'a pas pu ouvrir son affichage. {e}"));
        }
    }
}
