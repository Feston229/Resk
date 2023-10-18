import 'dart:ui';

import 'package:flutter/material.dart';
import 'package:resk_ui/main.dart';

class AddDevicePage extends StatefulWidget {
  const AddDevicePage({super.key});

  @override
  State<AddDevicePage> createState() => AddDevicePageState();
}

class AddDevicePageState extends State<AddDevicePage> {
  final newIdController = TextEditingController();

  @override
  void dispose() {
    super.dispose();
    newIdController.dispose();
  }

  void onPressed() {
    log.i(newIdController.text);
    newIdController.clear();
  }

  Widget addDeviceWidget(BuildContext context) {
    return Container(
      key: const Key('Add Device By Id'),
      child: Column(
        children: [
          const Align(
            alignment: Alignment.topLeft,
            child: Text(
              'Add device by id',
              style: TextStyle(fontSize: 20),
            ),
          ),
          Row(
            children: [
              Expanded(
                  child: TextField(
                controller: newIdController,
                decoration: const InputDecoration(
                    border: OutlineInputBorder(), hintText: 'Enter peer id'),
              )),
              SizedBox(
                  height: 50,
                  child: ElevatedButton(
                    onPressed: onPressed,
                    style: ButtonStyle(
                      shape: MaterialStateProperty.all<RoundedRectangleBorder>(
                          const RoundedRectangleBorder(
                              borderRadius: BorderRadius.zero)),
                    ),
                    child: const Text(
                      'OK',
                      style: TextStyle(color: Colors.white),
                    ),
                  ))
            ],
          )
        ],
      ),
    );
  }

  Widget selectLocalPeersWidget(BuildContext context) {
    return Container(
      padding: const EdgeInsetsDirectional.only(top: 10),
      child: const Text(
        'Or select from local network',
        style: TextStyle(fontSize: 20),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    Color secondaryColor = Theme.of(context).colorScheme.secondary;
    return Container(
        color: secondaryColor,
        padding: const EdgeInsets.symmetric(horizontal: 30, vertical: 40),
        child: Align(
            alignment: Alignment.center,
            child: Column(
              key: const Key('Add Device Column'),
              mainAxisAlignment: MainAxisAlignment.start,
              children: [
                addDeviceWidget(context),
                selectLocalPeersWidget(context),
              ],
            )));
  }
}

class _NewIdInput extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return const Text('input');
  }
}
