const fs = require("fs");
const path = require("path");
const zlib = require("zlib");

// Helper to write CRC32 for PNG chunks
function crc32(buf) {
  let table = [];
  for (let n = 0; n < 256; n++) {
    let c = n;
    for (let k = 0; k < 8; k++) {
      c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
    }
    table[n] = c;
  }
  let crc = 0 ^ -1;
  for (let i = 0; i < buf.length; i++) {
    crc = (crc >>> 8) ^ table[(crc ^ buf[i]) & 0xff];
  }
  return (crc ^ -1) >>> 0;
}

function makeChunk(type, data) {
  const len = Buffer.alloc(4);
  len.writeUInt32BE(data.length, 0);
  const typeAndData = Buffer.concat([Buffer.from(type), data]);
  const crc = Buffer.alloc(4);
  crc.writeUInt32BE(crc32(typeAndData), 0);
  return Buffer.concat([len, typeAndData, crc]);
}

function createPng(width, height) {
  const header = Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]);

  const ihdr = Buffer.alloc(13);
  ihdr.writeUInt32BE(width, 0);
  ihdr.writeUInt32BE(height, 4);
  ihdr.writeUInt8(8, 8); // 8-bit depth
  ihdr.writeUInt8(6, 9); // RGBA color
  ihdr.writeUInt8(0, 10);
  ihdr.writeUInt8(0, 11);
  ihdr.writeUInt8(0, 12);

  const rawRows = [];
  const cx = width / 2;
  const cy = height / 2;
  const r = width * 0.42;

  for (let y = 0; y < height; y++) {
    const row = Buffer.alloc(1 + width * 4);
    row.writeUInt8(0, 0); // Filter type None
    for (let x = 0; x < width; x++) {
      const dx = x - cx;
      const dy = y - cy;
      const dist = Math.sqrt(dx * dx + dy * dy);

      // EyeRest calm sage/teal icon: smooth circular eye iris
      if (dist <= r) {
        // Soft teal: #5EA89B -> R: 94, G: 168, B: 155
        const alpha = dist > r - 1.5 ? Math.floor(((r - dist) / 1.5) * 255) : 255;
        row.writeUInt8(94, 1 + x * 4);
        row.writeUInt8(168, 1 + x * 4 + 1);
        row.writeUInt8(155, 1 + x * 4 + 2);
        row.writeUInt8(alpha, 1 + x * 4 + 3);
      } else {
        // Transparent
        row.writeUInt32BE(0, 1 + x * 4);
      }
    }
    rawRows.push(row);
  }

  const rawData = Buffer.concat(rawRows);
  const compressed = zlib.deflateSync(rawData);

  const ihdrChunk = makeChunk("IHDR", ihdr);
  const idatChunk = makeChunk("IDAT", compressed);
  const iendChunk = makeChunk("IEND", Buffer.alloc(0));

  return Buffer.concat([header, ihdrChunk, idatChunk, iendChunk]);
}

// Minimal single-image ICO container
function createIco(pngBuffer, size) {
  const icoHeader = Buffer.alloc(6);
  icoHeader.writeUInt16LE(0, 0); // Reserved
  icoHeader.writeUInt16LE(1, 2); // Type 1 = ICO
  icoHeader.writeUInt16LE(1, 4); // 1 Image

  const icoEntry = Buffer.alloc(16);
  icoEntry.writeUInt8(size >= 256 ? 0 : size, 0); // Width
  icoEntry.writeUInt8(size >= 256 ? 0 : size, 1); // Height
  icoEntry.writeUInt8(0, 2); // Color palette
  icoEntry.writeUInt8(0, 3); // Reserved
  icoEntry.writeUInt16LE(1, 4); // Color planes
  icoEntry.writeUInt16LE(32, 6); // Bits per pixel
  icoEntry.writeUInt32LE(pngBuffer.length, 8); // Size
  icoEntry.writeUInt32LE(6 + 16, 12); // Offset

  return Buffer.concat([icoHeader, icoEntry, pngBuffer]);
}

const iconsDir = path.join(__dirname, "src-tauri", "icons");
if (!fs.existsSync(iconsDir)) fs.mkdirSync(iconsDir, { recursive: true });

const png32 = createPng(32, 32);
const png128 = createPng(128, 128);
const ico = createIco(png32, 32);

fs.writeFileSync(path.join(iconsDir, "32x32.png"), png32);
fs.writeFileSync(path.join(iconsDir, "128x128.png"), png128);
fs.writeFileSync(path.join(iconsDir, "icon.ico"), ico);

console.log("Generated 32x32.png, 128x128.png, and icon.ico successfully!");
