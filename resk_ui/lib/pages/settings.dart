import 'dart:io';

import 'package:flutter/material.dart';
import 'package:resk_ui/main.dart';
import 'package:settings_ui/settings_ui.dart';
import 'package:shared_preferences/shared_preferences.dart';
import 'package:file_picker/file_picker.dart';

class SettingsPage extends StatefulWidget {
  const SettingsPage({super.key});

  @override
  State<SettingsPage> createState() => SettingsPageState();
}

class SettingsPageState extends State<SettingsPage> {
  Directory? _syncDir;
  SharedPreferences? _preferences;
  @override
  void initState() {
    super.initState();
    _loadData();
  }

  Future<void> _loadData() async {
    SharedPreferences preferences = await SharedPreferences.getInstance();
    Directory syncDir = Directory(preferences.getString('syncDir') ??
        '${preferences.getString('rootDir')!}/Resk');
    setState(() {
      _syncDir = syncDir;
      _preferences = preferences;
    });
  }

  Future<void> _directoryPicker(BuildContext context) async {
    String? selectedDirectory = await FilePicker.platform
        .getDirectoryPath(initialDirectory: _syncDir?.path);
    if (selectedDirectory != null) {
      setState(() {
        _preferences?.setString('syncDir', selectedDirectory);
        _syncDir = Directory(selectedDirectory);
      });
    }
    log.i(selectedDirectory);
  }

  @override
  Widget build(BuildContext context) {
    Color secondaryColor = Theme.of(context).colorScheme.secondary;
    return Container(
      color: secondaryColor,
      child: Align(
        alignment: Alignment.center,
        child: SettingsList(
            darkTheme: const SettingsThemeData(
                settingsListBackground: Colors.transparent,
                settingsSectionBackground: Colors.transparent),
            sections: [
              SettingsSection(
                title: const Text('General'),
                tiles: <SettingsTile>[
                  SettingsTile.navigation(
                    title: const Text('Language'),
                    value: const Text('English'),
                    leading: const Icon(Icons.language),
                  ),
                  SettingsTile.navigation(
                    title: const Text('Sync location'),
                    value: Text(_syncDir?.path ?? ''),
                    leading: const Icon(Icons.sync),
                    onPressed: (context) {
                      _directoryPicker(context);
                    },
                  )
                ],
              )
            ]),
      ),
    );
  }
}
