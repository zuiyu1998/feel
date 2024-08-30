import 'package:json_annotation/json_annotation.dart';

part 'model.g.dart';

@JsonSerializable(genericArgumentFactories: true)
class PageList<T> {
  List<T> data;
  @JsonKey(name: "has_next")
  bool hasNext;
  int page;
  @JsonKey(name: "page_size")
  int pageSize;

  PageList(this.data, this.hasNext, this.page, this.pageSize);

  factory PageList.fromJson(
    Map<String, dynamic> json,
    T Function(dynamic json) fromJsonT,
  ) =>
      _$PageListFromJson<T>(json, fromJsonT);

  Map<String, dynamic> toJson(Object? Function(T value) toJsonT) =>
      _$PageListToJson<T>(this, toJsonT);
}
