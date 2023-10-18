import 'dart:io';

import 'package:device_info_plus/device_info_plus.dart';
import 'package:flash/flash_helper.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

class ConnectivityPage extends StatefulWidget {
  const ConnectivityPage({super.key});

  @override
  State<ConnectivityPage> createState() => ConnectivityPageState();
}

class ConnectivityPageState extends State<ConnectivityPage> {
  final Future<Map<String, dynamic>> _deviceData =
      Future<Map<String, dynamic>>(() async {
    Map<String, dynamic> response = {};

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
    } else {
      icon = const Icon(Icons.phone_iphone);
      hostnameLabel = const Text('ios');
    }
    response['icon'] = icon;
    response['hostnameLabel'] = hostnameLabel;

    const peerId = '123' /*await sendMsgNode('local_peer_id')*/;
    response['localPeerId'] = peerId;

    return response;
  });

  @override
  Widget build(BuildContext context) {
    Color secondaryColor = Theme.of(context).colorScheme.secondary;
    return FutureBuilder<Map<String, dynamic>>(
        future: _deviceData,
        builder: (BuildContext builder,
            AsyncSnapshot<Map<String, dynamic>> snapshot) {
          if (snapshot.hasData) {
            Widget icon = snapshot.data!['icon']!;
            Widget hostnameLabel = snapshot.data!['hostnameLabel']!;
            String localPeerId = snapshot.data!['localPeerId']!;

            return Container(
              color: secondaryColor,
              padding: const EdgeInsets.symmetric(horizontal: 30, vertical: 40),
              child: Align(
                alignment: Alignment.topLeft,
                child: Column(children: [
                  Padding(
                    padding: const EdgeInsets.only(bottom: 10),
                    child: Row(
                      children: [icon, hostnameLabel],
                    ),
                  ),
                  Padding(
                      padding: const EdgeInsets.only(bottom: 10),
                      child: Row(
                        children: [_PeerIdWidget(peerId: localPeerId)],
                      )),
                ]),
              ),
            );
          } else {
            return const Align(
              alignment: Alignment.center,
              child: CircularProgressIndicator(
                color: Colors.white,
              ),
            );
          }
        });
  }
}

class _PeerIdWidget extends StatelessWidget {
  final String peerId;

  const _PeerIdWidget({required this.peerId});

  void _copyToClipboard(BuildContext context) {
    Clipboard.setData(ClipboardData(text: peerId));
    context.showToast(const Text('Local id has been copied to clipboard'),
        backgroundColor: Colors.white,
        textStyle: const TextStyle(color: Colors.black));
  }

  @override
  Widget build(BuildContext context) {
    const double fontSize = 20;
    return GestureDetector(
      onTap: () => _copyToClipboard(context),
      child: const Text.rich(TextSpan(
          text: 'Local id: ',
          style: TextStyle(fontSize: fontSize),
          children: [
            TextSpan(
                text: '(Click to copy)',
                style: TextStyle(
                    decoration: TextDecoration.underline,
                    color: Colors.blue,
                    fontSize: fontSize))
          ])),
    );
  }
}
