// ignore: avoid_web_libraries_in_flutter, deprecated_member_use
import 'dart:html' as html;

void downloadFileFromUrl(String url, {String? filename}) {
  final anchor = html.AnchorElement(href: url)
    ..target = '_blank'
    ..download = filename ?? 'ecoalerta_reportes_pasco.csv';
  html.document.body?.children.add(anchor);
  anchor.click();
  anchor.remove();
}
