//! La lecture de l'écriture par ML Kit Digital Ink (Google, la technologie de Gboard), sur Android
//! (ADR 0012, exception à la GPL accordée par l'autrice). Le Java (`LecteurEncre`) fait le travail ;
//! ici : les appels et les réponses, qui arrivent sur le fil de l'interface.

use std::sync::Mutex;

use jni::JNIEnv;
use jni::objects::{JClass, JObjectArray, JString, JValue};
use jni::sys::jlong;

use crate::imp::{jstring, wake, with_activity};

/// Les lectures revenues : (identifiant, candidats du plus probable au moins probable, ou erreur).
static LUES: Mutex<Vec<(u64, Result<Vec<String>, String>)>> = Mutex::new(Vec::new());

/// L'état du modèle d'une langue : 0 absent, 1 téléchargement, 2 prêt, 3 erreur, -1 ML Kit indisponible.
pub fn etat(langue: &str) -> i32 {
    with_activity(|env, act| {
        let l = env.new_string(langue)?;
        env.call_method(act, "encreEtat", "(Ljava/lang/String;)I", &[JValue::Object(&l)])?
            .i()
    })
    .unwrap_or(-1)
}

/// Lance le téléchargement du modèle d'une langue (≈ 20 Mo, une fois).
pub fn telecharger(langue: &str) {
    with_activity(|env, act| {
        let l = env.new_string(langue)?;
        env.call_method(
            act,
            "encreTelecharger",
            "(Ljava/lang/String;)V",
            &[JValue::Object(&l)],
        )?;
        Ok(())
    });
}

/// Lit des traits : `traits` = liste de (x, y, t en ms) par trait ; `zone` = taille de la zone
/// d'écriture ; `avant` = le texte qui précède (il aide à lire).
pub fn lire(id: u64, langue: &str, traits: &[Vec<(f32, f32, i64)>], zone: (f32, f32), avant: &str) -> bool {
    let xs: Vec<f32> = traits.iter().flatten().map(|p| p.0).collect();
    let ys: Vec<f32> = traits.iter().flatten().map(|p| p.1).collect();
    let ts: Vec<i64> = traits.iter().flatten().map(|p| p.2).collect();
    let mut fins = Vec::with_capacity(traits.len());
    let mut n = 0i32;
    for t in traits {
        n += t.len() as i32;
        fins.push(n);
    }
    with_activity(|env, act| {
        let l = env.new_string(langue)?;
        let a = env.new_string(avant)?;
        let jx = env.new_float_array(xs.len() as i32)?;
        env.set_float_array_region(&jx, 0, &xs)?;
        let jy = env.new_float_array(ys.len() as i32)?;
        env.set_float_array_region(&jy, 0, &ys)?;
        let jt = env.new_long_array(ts.len() as i32)?;
        env.set_long_array_region(&jt, 0, &ts)?;
        let jf = env.new_int_array(fins.len() as i32)?;
        env.set_int_array_region(&jf, 0, &fins)?;
        env.call_method(
            act,
            "encreLire",
            "(JLjava/lang/String;[F[F[J[IFFLjava/lang/String;)Z",
            &[
                JValue::Long(id as jlong),
                JValue::Object(&l),
                JValue::Object(&jx),
                JValue::Object(&jy),
                JValue::Object(&jt),
                JValue::Object(&jf),
                JValue::Float(zone.0),
                JValue::Float(zone.1),
                JValue::Object(&a),
            ],
        )?
        .z()
    })
    .unwrap_or(false)
}

/// Les lectures revenues depuis le dernier appel.
pub fn lues() -> Vec<(u64, Result<Vec<String>, String>)> {
    LUES.lock()
        .map(|mut v| std::mem::take(&mut *v))
        .unwrap_or_default()
}

/// `FanoActivity.nativeOnEncreLue(long id, String[] candidats, String erreur)`.
#[unsafe(no_mangle)]
pub extern "system" fn Java_mg_fanoratana_shell_FanoActivity_nativeOnEncreLue(
    mut env: JNIEnv,
    _class: JClass,
    id: jlong,
    candidats: JObjectArray,
    erreur: JString,
) {
    let r = if erreur.is_null() {
        let n = env.get_array_length(&candidats).unwrap_or(0);
        let mut v = Vec::new();
        for i in 0..n {
            if let Ok(o) = env.get_object_array_element(&candidats, i) {
                if let Some(s) = jstring(&mut env, &JString::from(o)) {
                    v.push(s);
                }
            }
        }
        Ok(v)
    } else {
        Err(jstring(&mut env, &erreur).unwrap_or_default())
    };
    if let Ok(mut l) = LUES.lock() {
        l.push((id as u64, r));
    }
    wake();
}

/// `FanoActivity.nativeOnEncreModele()` : l'état d'un modèle a changé.
#[unsafe(no_mangle)]
pub extern "system" fn Java_mg_fanoratana_shell_FanoActivity_nativeOnEncreModele(
    _env: JNIEnv,
    _class: JClass,
) {
    wake();
}
