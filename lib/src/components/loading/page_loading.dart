import 'package:flutter/material.dart';
import 'package:flutter_spinkit/flutter_spinkit.dart';

class PageLoading extends StatelessWidget {
  const PageLoading({super.key});

  @override
  Widget build(BuildContext context) {
    return const SpinKitCircle(
      color: Colors.blue,
      size: 50.0,
    );
  }
}
