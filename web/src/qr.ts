// Matrice d'un code QR : la bibliothèque `qrcode-generator` calcule quelles cases sont noires, et l'interface
// les dessine elle-même en SVG (components/QrCode.tsx), sans insérer de HTML produit par la bibliothèque.
import qrcode from "qrcode-generator";

/** Cases d'un code QR (`true` = case sombre), ligne par ligne. Taille choisie automatiquement (type 0) ;
 * correction d'erreurs « M » (15 % de la surface peut être abîmée), le niveau courant pour un lien. */
export function qrMatrix(text: string): boolean[][] {
  const qr = qrcode(0, "M");
  qr.addData(text);
  qr.make();
  const size = qr.getModuleCount();
  return Array.from({ length: size }, (_, row) => Array.from({ length: size }, (_, col) => qr.isDark(row, col)));
}
