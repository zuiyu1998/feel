import 'package:flutter/material.dart';

class ImagesView extends StatelessWidget {
  const ImagesView({super.key, required this.images});

  final List<String> images;

  Widget oneImage(double screenWidth) {
    return Container(
      margin: const EdgeInsets.fromLTRB(0.0, 10.0, 0.0, 10.0),
      width: screenWidth - 32,
      child: AspectRatio(
        aspectRatio: 1.0,
        child: Image.network(images[0]),
      ),
    );
  }

  Widget twoImage(double screenWidth) {
    final imageWidth = (screenWidth - 32 - 10) / 2.0;

    return Container(
      margin: const EdgeInsets.fromLTRB(0.0, 10.0, 0.0, 10.0),
      width: screenWidth - 32,
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          SizedBox(
            width: imageWidth,
            child: AspectRatio(
              aspectRatio: 1.0,
              child: Image.network(images[0]),
            ),
          ),
          SizedBox(
            width: imageWidth,
            child: AspectRatio(
              aspectRatio: 1.0,
              child: Image.network(images[1]),
            ),
          )
        ],
      ),
    );
  }

  Widget moreImage(double screenWidth) {
    final imageWidth = (screenWidth - 32 - 20) / 3.0;

    final List<Widget> list = [];

    for (var image in images) {
      list.add(Container(
        margin: const EdgeInsets.fromLTRB(0.0, 10.0, 0.0, 0.0),
        width: imageWidth,
        child: AspectRatio(
          aspectRatio: 1.0,
          child: Image.network(image),
        ),
      ));
    }

    return Container(
      width: screenWidth,
      margin: const EdgeInsets.fromLTRB(0.0, 0.0, 0.0, 10.0),
      child: Wrap(
        crossAxisAlignment: WrapCrossAlignment.start,
        alignment: WrapAlignment.spaceBetween,
        children: list,
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final size = MediaQuery.of(context).size;
    final width = size.width;

    switch (images.length) {
      case 1:
        return oneImage(width);

      case 2:
        return twoImage(width);
      default:
        return moreImage(width);
    }
  }
}
