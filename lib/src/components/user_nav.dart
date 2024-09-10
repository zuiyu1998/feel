import 'package:feel/src/apis/post/model.dart';
import 'package:feel/src/apis/user/model.dart';
import 'package:flutter/material.dart';

class UserNav extends StatefulWidget {
  final UserBaseModel? user;
  final void Function()? goTo;

  const UserNav({super.key, this.user, this.goTo});

  @override
  State<UserNav> createState() => _UserNavState();
}

class _UserNavState extends State<UserNav> {
  late Post post;

  @override
  Widget build(BuildContext context) {
    return SizedBox(
      height: 48,
      child: Row(
        children: [
          InkWell(
            onTap: () {
              if (widget.goTo != null) {
                widget.goTo!();
              }
            },
            child: const Icon(
              Icons.navigate_before,
              size: 36,
            ),
          )
        ],
      ),
    );
  }
}
