# Appelées depuis Rust par JNI, par leur nom.
-keep class mg.fanoratana.shell.FanoActivity { *; }
-keep class mg.fanoratana.shell.ApkInstaller { *; }
-keep class com.google.androidgamesdk.** { *; }
-keepclasseswithmembernames class mg.fanoratana.shell.** { native <methods>; }

# ML Kit tire WorkManager, dont la base Room est créée par réflexion (`WorkDatabase_Impl`) : sans cette règle,
# R8 la retire et l'app plante au démarrage (Pad 7, 2026-11-08).
-keep class * extends androidx.room.RoomDatabase { <init>(); }
-keep class androidx.work.impl.** { *; }

# ML Kit s'initialise par réflexion (ComponentDiscovery) : R8 retirait le constructeur sans argument des
# registres (`DigitalInkRecognitionRegistrar`, `CommonComponentRegistrar`), ML Kit ne démarrait pas et le
# modèle Google ne se téléchargeait jamais (Pad 7, 2026-11-09).
-keep class * implements com.google.firebase.components.ComponentRegistrar { public <init>(); }
-keep class com.google.mlkit.** { *; }
