import 'dart:io';

import 'package:feel/mock/index.dart';
import 'package:feel/src/router/index.dart';
import 'package:feel/src/utils/http/index.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:get/get.dart';

void main() async {
  if (kDebugMode) {
    startMockDio(defineDio.dio);
  }

  WidgetsFlutterBinding.ensureInitialized();

  await bootstrap();

  runApp(const MyApp());
}

Future<void> bootstrap() async {
  //状态栏透明
  updateStatsBar();
}

void updateStatsBar() {
  if (Platform.isAndroid) {
    //设置Android头部的导航栏透明
    SystemUiOverlayStyle systemUiOverlayStyle = const SystemUiOverlayStyle(
      statusBarColor: Colors.transparent, //全局设置透明
    );
    SystemChrome.setSystemUIOverlayStyle(systemUiOverlayStyle);
  }
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
