import 'dart:html' as html;

void changeFavicon(String url) {
  try {
    final link = html.document.querySelector("link[rel*='icon']") as html.LinkElement?;
    if (link != null) {
      link.href = url;
    } else {
      final newLink = html.LinkElement()
        ..rel = 'icon'
        ..href = url;
      html.document.head?.append(newLink);
    }
  } catch (e) {
    // ignore
  }
}

void openUrl(String url) {
  try {
    html.window.open(url, '_blank');
  } catch (e) {
    // ignore
  }
}
