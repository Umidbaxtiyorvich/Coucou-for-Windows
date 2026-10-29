package com.coucou.android.app

import android.app.Application
import android.content.Context
import com.coucou.android.core.Languages
import com.coucou.android.core.Texts

class CoucouApp : Application() {
    val model: AppModel by lazy { AppModel(this) }

    override fun attachBaseContext(base: Context) {
        // Before Android 13 the language chosen in the app is applied here (from 13 on the system does it).
        super.attachBaseContext(AppLanguage.wrap(base))
    }

    override fun onCreate() {
        super.onCreate()
        // This build speaks Uzbek: on the first launch the app's language is set to it once. Settings > Language changes it after.
        val prefs = getSharedPreferences("coucou_ui", MODE_PRIVATE)
        if (!prefs.getBoolean("uz_default_applied", false)) {
            prefs.edit().putBoolean("uz_default_applied", true).apply()
            if (AppLanguage.current(this) == Languages.SYSTEM) AppLanguage.set(this, "uz")
        }
        // The words of the pure rules follow the app's language like every other text.
        Texts.source = AppLanguage.texts(this)
    }
}
