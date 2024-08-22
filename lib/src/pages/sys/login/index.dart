import 'package:flutter/material.dart';

class LoginPage extends StatefulWidget {
  const LoginPage({
    super.key,
  });

  @override
  State<LoginPage> createState() => _LoginPageState();
}

class _LoginPageState extends State<LoginPage> {
  final _formKey = GlobalKey<FormState>();

  @override
  Widget build(BuildContext context) {
    return Scaffold(
        body: Form(
            key: _formKey,
            child: Container(
              margin: const EdgeInsets.fromLTRB(0.0, 80.0, 0.0, 0.0),
              child: Column(
                children: [
                  Container(
                    margin: const EdgeInsets.fromLTRB(0.0, 0.0, 0.0, 32.0),
                    decoration: const BoxDecoration(
                        // 下滑线浅灰色，宽度1像素
                        border: Border(
                            bottom:
                                BorderSide(color: Colors.grey, width: 1.0))),
                    child: const TextField(
                        keyboardType: TextInputType.emailAddress,
                        decoration: InputDecoration(
                            prefixIcon: Icon(Icons.email),
                            border: InputBorder.none //隐藏下划线
                            )),
                  ),
                  Container(
                    decoration: const BoxDecoration(
                        // 下滑线浅灰色，宽度1像素
                        border: Border(
                            bottom:
                                BorderSide(color: Colors.grey, width: 1.0))),
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
                              if ((_formKey.currentState as FormState)
                                  .validate()) {
                                //验证通过提交数据
                                debugPrint("test");
                              }
                            },
                          ),
                        ),
                      ],
                    ),
                  )
                ],
              ),
            )));
  }
}
