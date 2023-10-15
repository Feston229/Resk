import 'dart:io';

import 'package:device_info_plus/device_info_plus.dart';
import 'package:flutter/material.dart';
import 'package:logger/logger.dart';
import 'package:animations/animations.dart';
import 'package:resk_ui/controllers.dart';
import 'package:resk_ui/pages/connectivity.dart';
import 'package:resk_ui/pages/add_device.dart';
import 'package:resk_ui/pages/settings.dart';
import 'dart:async';

import 'package:flash/flash.dart';
import 'package:flash/flash_helper.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';

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
/*       body: Center(
        child: PageTransitionSwitcher(
          transitionBuilder: (child, animation, secondaryAnimation) {
            return FadeThroughTransition(
              animation: animation,
              secondaryAnimation: secondaryAnimation,
              child: child,
            );
          },
          child: _PageNavigator(
              key: UniqueKey(),
              item: bottomNavigationBarItems[_currentIndex.value]),
        ),
      ), */
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

class _NavigationDestinationView extends StatefulWidget {
  const _NavigationDestinationView({
    super.key,
    required this.item,
  });

  final BottomNavigationBarItem item;
  @override
  State<_NavigationDestinationView> createState() =>
      _NavigationDestinationViewState();
}

class _NavigationDestinationViewState
    extends State<_NavigationDestinationView> {
  BottomNavigationBarItem get item => widget.item;

  Widget _connectivityPage(Color mainColor) {
    return Container(
        color: mainColor,
        padding:
            const EdgeInsets.only(left: 40, top: 30, right: 40, bottom: 30),
        child: FutureBuilder<Map<String, Widget>>(
          future: _deviceData,
          builder: (BuildContext context,
              AsyncSnapshot<Map<String, Widget>> snapshot) {
            Widget child;
            if (snapshot.hasData) {
              Widget icon = snapshot.data!['icon']!;
              Widget hostnameLabel = snapshot.data!['hostnameLabel']!;
              Widget localPeerId = snapshot.data!['localPeerId']!;
              child = Align(
                alignment: Alignment.topLeft,
                child: Column(children: [
                  Row(children: [icon, hostnameLabel]),
                  Row(children: [localPeerId])
                ]),
              );
            } else if (snapshot.hasError) {
              child = const Text('hasError');
            } else {
              child = const Align(
                alignment: Alignment.center,
                child: SizedBox(
                  width: 40,
                  height: 40,
                  child: CircularProgressIndicator(),
                ),
              );
            }
            return child;
          },
        ));
  }

  Widget _addDevicePage(Color mainColor) {
    return Container(
      color: mainColor,
      child: const Align(
        alignment: Alignment.topLeft,
        child: Row(children: [Text('Add Device')]),
      ),
    );
  }

  Widget _settingsPage(Color mainColor) {
    return Container(
      color: mainColor,
      child: const Align(
        alignment: Alignment.topLeft,
        child: Row(children: [Text('Settings')]),
      ),
    );
  }

  final Future<Map<String, Widget>> _deviceData =
      Future<Map<String, Widget>>(() async {
    log.i('deviceData started');
    Map<String, Widget> response = {};

    Icon icon;
    Text hostnameLabel;

    DeviceInfoPlugin deviceInfo = DeviceInfoPlugin();
    if (Platform.isAndroid) {
      icon = const Icon(
        Icons.phone_android,
        size: 45,
      );
      AndroidDeviceInfo androidInfo = await deviceInfo.androidInfo;
      hostnameLabel = Text(
        androidInfo.model,
        style: const TextStyle(fontSize: 22),
      );
    } else if (Platform.isIOS) {
      icon = const Icon(Icons.phone_iphone);
      hostnameLabel = const Text('ios');
    } else {
      icon = const Icon(Icons.laptop);
      hostnameLabel = const Text('somick');
    }
    response['icon'] = icon;
    response['hostnameLabel'] = hostnameLabel;

    final peerId = await sendMsgNode('local_peer_id');
    response['localPeerId'] = Text(
      peerId,
      style: const TextStyle(fontSize: 8),
    );

    return response;
  });

  @override
  Widget build(BuildContext context) {
    final Color mainColor = Theme.of(context).colorScheme.secondary;
    Widget? page;
    switch (item.label) {
      case CONNECTIVITY_LABEL:
        log.i('start');
        page = _connectivityPage(mainColor);
        log.i('finish');
      case ADD_DEVICE_LABEL:
        log.i('start');
        page = _addDevicePage(mainColor);
        log.i('finish');
      case SETTINGS_LABEL:
        page = _settingsPage(mainColor);
    }
    return page!;
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
