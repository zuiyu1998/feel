import 'package:feel/mock/index.dart';
import 'package:feel/src/router/index.dart';
import 'package:feel/src/utils/http/index.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:get/get.dart';

void main() {
  if (kDebugMode) {
    startMockDio(defineDio.dio);
  }

  runApp(const MyApp());
}

class MyApp extends StatelessWidget {
  const MyApp({super.key});

  @override
  Widget build(BuildContext context) {
    return GetMaterialApp(
      initialRoute: RouterBuilder.initial,
      getPages: RouterBuilder.routes,
    );
  }
}
