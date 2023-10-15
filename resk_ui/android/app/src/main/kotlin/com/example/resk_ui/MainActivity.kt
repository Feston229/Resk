package com.example.resk_ui

import android.os.Environment
import androidx.annotation.NonNull
import io.flutter.embedding.android.FlutterActivity
import io.flutter.embedding.engine.FlutterEngine
import io.flutter.plugin.common.MethodChannel

class MainActivity : FlutterActivity() {
    private val CHANNEL = "resk_channel"

    override fun configureFlutterEngine(@NonNull flutterEngine: FlutterEngine) {
        super.configureFlutterEngine(flutterEngine)
        MethodChannel(flutterEngine.dartExecutor.binaryMessenger, CHANNEL).setMethodCallHandler {
                call,
                result ->
            when (call.method) {
                "getRootDir" -> {
                    val response = getRootDir()
                    result.success(response)
                }
                else -> {
                    result.notImplemented()
                }
            }
        }
    }

    private fun getRootDir(): String {
        val rootDirectory: String = Environment.getExternalStorageDirectory().absolutePath
        return rootDirectory
    }
}
