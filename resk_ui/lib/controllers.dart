import 'dart:async';
import 'dart:convert';
import 'dart:ffi';
import 'dart:io';
import 'dart:isolate';
import 'dart:ui';

import 'package:flutter/services.dart';
import 'package:flutter_background_service/flutter_background_service.dart';
import 'package:flutter_local_notifications/flutter_local_notifications.dart';
import 'package:path_provider/path_provider.dart';
import 'package:resk_ui/communication.dart';
import 'package:permission_handler/permission_handler.dart';
import 'package:resk_ui/main.dart';
import 'package:shared_preferences/shared_preferences.dart';

typedef ReskNodeC = Void Function(Int32);
typedef ReskNodeDart = void Function(int);
const notificationChannelId = 'resk_foreground';
const notificationId = 797;

Future<void> runNode(Map<String, dynamic> dataSet) async {
  // Locate library
  String libPath;
  switch (Platform.operatingSystem) {
    case 'android':
      libPath = 'libresk_android.so';
    case _:
      libPath = '';
  }
  final DynamicLibrary rustLib = DynamicLibrary.open(libPath);

  // Start udp listener for communications
  final ReceivePort listenRustUdpReceivePort = ReceivePort();
  dataSet['sendPort'] = listenRustUdpReceivePort.sendPort;
  Isolate.spawn(listenRustRequestsUdp, dataSet);
  int udpPort = await listenRustUdpReceivePort.first;

  // Run resk node
  final reskNode =
      rustLib.lookupFunction<ReskNodeC, ReskNodeDart>('run_node_android');
  reskNode(udpPort);
}

Future<void> requestPermissions() async {
  var storageStatus = await Permission.storage.request();
  if (storageStatus.isGranted) {
    log.i('Storage permissions has been granted');
  } else {
    log.i('Storage permissions has been denied');
  }

  var storageGlobalStatus = await Permission.manageExternalStorage.request();
  if (storageGlobalStatus.isGranted) {
    log.i('Global storage permissions has been granted');
  } else {
    log.i('Global storage permissions has been denied');
  }

  var notificationStatus = await Permission.notification.request();
  if (notificationStatus.isGranted) {
    log.i('Notification permissions has been granted');
  }
}

Future<Map<String, dynamic>> initDataSet(
    RootIsolateToken rootIsolateToken) async {
  Map<String, dynamic> dataSet = {};

  dataSet['rootIsolateToken'] = rootIsolateToken;

  return dataSet;
}

Future<void> initializeService() async {
  final service = FlutterBackgroundService();

  const AndroidNotificationChannel channel = AndroidNotificationChannel(
      notificationChannelId, 'Resk Foreground Service',
      description: 'This is Resk foreground channel',
      importance: Importance.low);
  final FlutterLocalNotificationsPlugin flutterLocalNotificationsPlugin =
      FlutterLocalNotificationsPlugin();
  await flutterLocalNotificationsPlugin
      .resolvePlatformSpecificImplementation<
          AndroidFlutterLocalNotificationsPlugin>()
      ?.createNotificationChannel(channel);

  flutterLocalNotificationsPlugin.initialize(
      const InitializationSettings(
          android: AndroidInitializationSettings('ic_bg_service_small')),
      onDidReceiveNotificationResponse: onSelectNotificationAction,
      onDidReceiveBackgroundNotificationResponse: onSelectNotificationAction);

  await service.configure(
      iosConfiguration: IosConfiguration(),
      androidConfiguration: AndroidConfiguration(
          foregroundServiceNotificationId: notificationId,
          onStart: onStart,
          autoStart: true,
          isForegroundMode: true,
          autoStartOnBoot: true));
}

Future<void> onStart(ServiceInstance service) async {
  DartPluginRegistrant.ensureInitialized();

  // Spawn node
  final RootIsolateToken rootIsolateToken = RootIsolateToken.instance!;
  Isolate.spawn(runNode, await initDataSet(rootIsolateToken));

  final FlutterLocalNotificationsPlugin flutterLocalNotificationsPlugin =
      FlutterLocalNotificationsPlugin();

  final List<AndroidNotificationAction> actions = [
    const AndroidNotificationAction('1', 'Send'),
    const AndroidNotificationAction('2', 'Get')
  ];

  // Show connected devices
  flutterLocalNotificationsPlugin.show(
      notificationId,
      'Resk',
      'You are not connected!',
      NotificationDetails(
          android: AndroidNotificationDetails(
              notificationChannelId, 'RESK FOREGROUND SERVICE',
              actions: actions,
              playSound: false,
              icon: 'ic_bg_service_small',
              ongoing: true)));
}

void onSelectNotificationAction(NotificationResponse response) async {
  log.i('actionId -> ${response.actionId}');
  log.i('payload -> ${response.payload}');
  log.i('input -> ${response.input}');
}

Future<void> initPlatformData() async {
  final SharedPreferences sharedPreferences =
      await SharedPreferences.getInstance();
  const channel = MethodChannel('resk_channel');
  final String rootDir = await channel.invokeMethod('getRootDir');
  sharedPreferences.setString('rootDir', rootDir);
}

Future<int> loadNodePort() async {
  final path = await getExternalStorageDirectory();
  final String pathStr = '${path!.path}/data.json';
  log.i(pathStr);
  Map<String, dynamic> jsonData =
      jsonDecode(await File(pathStr).readAsString());
  final int port = int.parse(jsonData['port']);
  return port;
}

Future<String> sendMsgNode(String msg) async {
  Completer<String> completer = Completer<String>();
  RawDatagramSocket socket = await RawDatagramSocket.bind('127.0.0.1', 0);

  socket.send(
      utf8.encode(msg), InternetAddress('127.0.0.1'), await loadNodePort());

  socket.listen((RawSocketEvent event) {
    if (event == RawSocketEvent.read) {
      Datagram? datagram = socket.receive();
      if (datagram != null) {
        completer.complete(utf8.decode(datagram.data));
        socket.close();
      }
    }
  });
  return completer.future;
}
