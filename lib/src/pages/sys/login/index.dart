import 'package:feel/src/assets/default.dart';
import 'package:feel/src/pages/home/index.dart';
import 'package:feel/src/store/modules/user.dart';
import 'package:flutter/material.dart';
import 'package:get/get.dart';

class LoginPage extends StatefulWidget {
  const LoginPage({
    super.key,
  });

  @override
  State<LoginPage> createState() => _LoginPageState();
}

class _LoginPageState extends State<LoginPage> {
  final _formKey = GlobalKey<FormState>();
  var userStore = Get.find<UserStore>();

  Future<void> submmit() async {
    await userStore.login();

    var token = userStore.getToken();

    if (token != null) {
      Get.off(const HomePage());
    }
  }

  Widget getForm() {
    return Form(
        key: _formKey,
        child: Column(
          children: [
            Container(
              margin: const EdgeInsets.fromLTRB(0.0, 32.0, 0.0, 0.0),
              decoration: const BoxDecoration(
                  // 下滑线浅灰色，宽度1像素
                  border: Border(
                      bottom: BorderSide(color: Colors.grey, width: 1.0))),
              child: const TextField(
                  keyboardType: TextInputType.emailAddress,
                  decoration: InputDecoration(
                      prefixIcon: Icon(Icons.email),
                      border: InputBorder.none //隐藏下划线
                      )),
            ),
            Container(
              margin: const EdgeInsets.fromLTRB(0.0, 32.0, 0.0, 0.0),
              decoration: const BoxDecoration(
                  // 下滑线浅灰色，宽度1像素
                  border: Border(
                      bottom: BorderSide(color: Colors.grey, width: 1.0))),
              child: const TextField(
                  keyboardType: TextInputType.emailAddress,
                  decoration: InputDecoration(
                      prefixIcon: Icon(Icons.email),
                      border: InputBorder.none //隐藏下划线
                      )),
            ),
            Padding(
              padding: const EdgeInsets.only(top: 28.0),
              child: Row(
                children: <Widget>[
                  Expanded(
                    child: ElevatedButton(
                      child: const Padding(
                        padding: EdgeInsets.all(16.0),
                        child: Text("登录"),
                      ),
                      onPressed: () {
                        if ((_formKey.currentState as FormState).validate()) {
                          submmit();
                        }
                      },
                    ),
                  ),
                ],
              ),
            )
          ],
        ));
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
        body: Column(
      children: [
        Container(
          margin: const EdgeInsets.fromLTRB(0.0, 80.0, 0.0, 0.0),
          child: AssetsDefault.defaultImage,
        ),
        getForm()
      ],
    ));
  }
}
