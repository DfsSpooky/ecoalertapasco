import 'dart:typed_data';
import 'package:image/image.dart' as img;

class ImageCompressor {
  /// Recibe los bytes de una imagen, la decodifica, la redimensiona si excede la
  /// dimensión máxima manteniendo su relación de aspecto, y la vuelve a codificar
  /// a JPG con la calidad especificada (por defecto 80%).
  static Uint8List compress(List<int> bytes, {int maxDimension = 1080, int quality = 80}) {
    try {
      final decodedImage = img.decodeImage(Uint8List.fromList(bytes));
      if (decodedImage == null) {
        return Uint8List.fromList(bytes);
      }

      img.Image resizedImage = decodedImage;
      if (decodedImage.width > maxDimension || decodedImage.height > maxDimension) {
        if (decodedImage.width > decodedImage.height) {
          resizedImage = img.copyResize(decodedImage, width: maxDimension);
        } else {
          resizedImage = img.copyResize(decodedImage, height: maxDimension);
        }
      }

      final compressedBytes = img.encodeJpg(resizedImage, quality: quality);
      return Uint8List.fromList(compressedBytes);
    } catch (_) {
      // Fallback a los bytes originales si ocurre algún error en el procesamiento
      return Uint8List.fromList(bytes);
    }
  }
}
