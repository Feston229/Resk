import 'dart:io';

import 'package:flutter/material.dart';
import 'package:logger/logger.dart';
import 'package:resk_ui/controllers.dart';
import 'package:resk_ui/pages/connectivity.dart';
import 'package:resk_ui/pages/add_device.dart';
import 'package:resk_ui/pages/settings.dart';
import 'dart:async';

import 'package:flash/flash_helper.dart';

final log = Logger();

const CONNECTIVITY_LABEL = 'Connectivity';
const ADD_DEVICE_LABEL = 'Add Device';
const SETTINGS_LABEL = 'Settings';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();

  // Mobile specific settings
  if (Platform.isAndroid || Platform.isIOS) {
    await requestPermissions();
    await initializeService();
    await initPlatformData();
  }

  // Run main app
  runApp(const App());
}

class App extends StatelessWidget {
  const App({super.key});

  static const ColorScheme colorScheme = ColorScheme.dark(
      primary: Color(0xFF1F1B24),
      secondary: Color.fromARGB(255, 39, 34, 46),
      onPrimary: Colors.white,
      onSecondary: Colors.white);
  // This widget is the root of your application.
  @override
  Widget build(BuildContext context) {
    final navigatorKey = GlobalKey<NavigatorState>();
    return MaterialApp(
      title: 'Resk',
      theme: ThemeData(
        colorScheme: colorScheme,
        useMaterial3: true,
      ),
      navigatorKey: navigatorKey,
      builder: (context, _) {
        var child = _!;
        // Wrap with toast.
        child = Toast(navigatorKey: navigatorKey, child: child);
        return child;
      },
      home: const HomePage(restorationId: '12', title: 'Resk'),
    );
  }
}

class HomePage extends StatefulWidget {
  const HomePage({super.key, required this.restorationId, required this.title});
  final String restorationId;
  final String title;

  @override
  State<HomePage> createState() => _HomePageState();
}

class _HomePageState extends State<HomePage> with RestorationMixin {
  final RestorableInt _currentIndex = RestorableInt(0);

  @override
  String get restorationId => widget.restorationId;

  @override
  void restoreState(RestorationBucket? oldBucket, bool initialRestore) {
    registerForRestoration(_currentIndex, 'bottom_navigation_tab_index');
  }

  @override
  void dispose() {
    _currentIndex.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final colorScheme = Theme.of(context).colorScheme;
    final textTheme = Theme.of(context).textTheme;

    var bottomNavigationBarItems = <BottomNavigationBarItem>[
      const BottomNavigationBarItem(
        icon: Icon(Icons.phonelink),
        label: CONNECTIVITY_LABEL,
      ),
      const BottomNavigationBarItem(
        icon: Icon(Icons.add),
        label: ADD_DEVICE_LABEL,
      ),
      const BottomNavigationBarItem(
        icon: Icon(Icons.settings),
        label: SETTINGS_LABEL,
      ),
    ];

    return Scaffold(
      appBar: AppBar(
        automaticallyImplyLeading: false,
        title: Text(
          widget.title,
          style: TextStyle(color: colorScheme.onPrimary),
        ),
        backgroundColor: colorScheme.primary,
      ),
      body: Center(
        child: _PageNavigator(
            key: UniqueKey(),
            item: bottomNavigationBarItems[_currentIndex.value]),
      ),
      bottomNavigationBar: Theme(
        data: Theme.of(context).copyWith(
            splashColor: Colors.transparent, highlightColor: Colors.white),
        child: BottomNavigationBar(
          showUnselectedLabels: true,
          items: bottomNavigationBarItems,
          currentIndex: _currentIndex.value,
          type: BottomNavigationBarType.fixed,
          selectedFontSize: textTheme.bodySmall!.fontSize!,
          unselectedFontSize: textTheme.bodySmall!.fontSize!,
          onTap: (index) {
            setState(() {
              _currentIndex.value = index;
            });
          },
          selectedItemColor: colorScheme.onPrimary,
          unselectedItemColor: colorScheme.onPrimary.withOpacity(0.38),
          backgroundColor: colorScheme.primary,
        ),
      ),
    );
  }
}

class _PageNavigator extends StatefulWidget {
  const _PageNavigator({super.key, required this.item});
  final BottomNavigationBarItem item;

  @override
  State<_PageNavigator> createState() => _PageNavigatorState();
}

class _PageNavigatorState extends State<_PageNavigator> {
  BottomNavigationBarItem get item => widget.item;

  final _pageMapping = <String, Widget>{
    CONNECTIVITY_LABEL: ConnectivityPage(
      key: UniqueKey(),
    ),
    ADD_DEVICE_LABEL: AddDevicePage(
      key: UniqueKey(),
    ),
    SETTINGS_LABEL: SettingsPage(
      key: UniqueKey(),
    ),
  };

  @override
  Widget build(BuildContext context) {
    final pageLabel = item.label!;
    return _pageMapping[pageLabel] ?? _pageMapping[CONNECTIVITY_LABEL]!;
  }
}
