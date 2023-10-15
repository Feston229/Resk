import 'dart:convert';
import 'dart:io';
import 'dart:isolate';

import 'package:path_provider/path_provider.dart';
import 'package:resk_ui/main.dart';
import 'package:flutter/services.dart';
import 'package:shared_preferences/shared_preferences.dart';

Future<void> listenRustRequestsUdp(Map<String, dynamic> dataSet) async {
  // Init code
  SendPort sendPort = dataSet['sendPort'];
  RootIsolateToken rootIsolateToken = dataSet['rootIsolateToken'];
  BackgroundIsolateBinaryMessenger.ensureInitialized(rootIsolateToken);
  final SharedPreferences sharedPreferences =
      await SharedPreferences.getInstance();

  try {
    RawDatagramSocket.bind('127.0.0.1', 0).then((RawDatagramSocket socket) {
      log.i('Listenig for udp requests on 127.0.0.1:${socket.port}');
      sendPort.send(socket.port);
      socket.listen((RawSocketEvent event) async {
        if (event == RawSocketEvent.read) {
          Datagram? dg = socket.receive();
          if (dg != null) {
            String message = String.fromCharCodes(dg.data);
            log.i('Got UDP request: $message');
            String responseStr;
            final List<String> requestList = message.split(':');
            switch (requestList.first) {
              case 'data_dir':
                final mobileDataPath = await getExternalStorageDirectory();
                responseStr = mobileDataPath?.path ?? '';
              case 'root_dir':
                final rootDir = sharedPreferences.getString('rootDir');
                responseStr = rootDir ?? '';
              case 'get_content':
                final clipboard = await Clipboard.getData(Clipboard.kTextPlain);
                responseStr = clipboard?.text ?? '';
              case 'set_content':
                final content = requestList.elementAt(1);
                Clipboard.setData(ClipboardData(text: content));
                responseStr = 'OK';
              case _:
                responseStr = '';
            }
            final responseBytes = utf8.encode(responseStr);
            socket.send(responseBytes, dg.address, dg.port);
            log.i('Replied with $responseStr');
          }
        }
      });
    });
  } catch (e) {
    log.e(e);
  }
}
