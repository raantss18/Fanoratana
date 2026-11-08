# Appelées depuis Rust par JNI, par leur nom.
-keep class mg.fanoratana.shell.FanoActivity { *; }
-keep class mg.fanoratana.shell.ApkInstaller { *; }
-keep class com.google.androidgamesdk.** { *; }
-keepclasseswithmembernames class mg.fanoratana.shell.** { native <methods>; }
