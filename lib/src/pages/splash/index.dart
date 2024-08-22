/*
加载界面用于从其他系统同步数据
- 文件系统
- 网络
 */

import 'package:feel/src/pages/home/index.dart';
import 'package:feel/src/pages/sys/login/index.dart';
import 'package:feel/src/store/index.dart';
import 'package:feel/src/store/modules/user.dart';
import 'package:flutter/material.dart';
import 'package:get/get.dart';

class SplashPage extends StatefulWidget {
  const SplashPage({
    super.key,
  });

  @override
  State<SplashPage> createState() => _SplashPageState();
}

class _SplashPageState extends State<SplashPage> {
  Future<void> bootstrap() async {
    ///全局实例注入
    ///从文件系统读取持久化数据
    await initStore();

    ///从网络同步数据

    ///更新路由
    ///
    await jumpToHomeOrLogin();
  }

  Future<void> jumpToHomeOrLogin() async {
    var userStore = Get.find<UserStore>();

    if (userStore.isAuth()) {
      await Get.off(const HomePage());
    } else {
      await Get.off(const LoginPage());
    }
  }

  @override
  void initState() {
    super.initState();

    WidgetsBinding.instance.addPostFrameCallback((_) {
      bootstrap();
    });
  }

  @override
  Widget build(BuildContext context) {
    return Container(
      color: Colors.red,
    );
  }
}
