import 'package:moment_dart/moment_dart.dart';

String fromNow(DateTime target) {
  final tmp = Moment(target);
  return tmp.fromNow();
}
