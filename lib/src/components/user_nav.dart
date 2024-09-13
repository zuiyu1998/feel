import 'package:feel/src/apis/user/model.dart';
import 'package:flutter/material.dart';

class UserNav extends StatefulWidget {
  final UserBaseModel user;
  final void Function()? goTo;
  final double opacity;

  const UserNav(
      {super.key, required this.user, required this.opacity, this.goTo});

  @override
  State<UserNav> createState() => _UserNavState();
}

class _UserNavState extends State<UserNav> {
  Widget user() {
    return Expanded(
      child: Opacity(
        opacity: widget.opacity,
        child: Row(
          children: [
            Image.network(
              widget.user.avatar,
              width: 32,
              height: 32,
            ),
            Expanded(
                child: Container(
              padding: const EdgeInsets.fromLTRB(8, 0, 0, 0),
              child: Column(
                children: [
                  Row(
                    children: [Text(widget.user.nikename)],
                  ),
                ],
              ),
            ))
          ],
        ),
      ),
    );
  }

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
          ),
          user()
        ],
      ),
    );
  }
}
