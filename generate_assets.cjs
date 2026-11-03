const fs = require("fs");
const path = require("path");

// 1. Generate gentle chime.wav
function generateChime() {
  const sampleRate = 44100;
  const durationSec = 2.5;
  const numSamples = Math.floor(sampleRate * durationSec);
  const numChannels = 1;
  const bytesPerSample = 2;
  const blockAlign = numChannels * bytesPerSample;
  const byteRate = sampleRate * blockAlign;
  const dataSize = numSamples * blockAlign;

  const buffer = Buffer.alloc(44 + dataSize);

  // RIFF header
  buffer.write("RIFF", 0);
  buffer.writeUInt32LE(36 + dataSize, 4);
  buffer.write("WAVE", 8);
  buffer.write("fmt ", 12);
  buffer.writeUInt32LE(16, 16);
  buffer.writeUInt16LE(1, 20); // PCM
  buffer.writeUInt16LE(numChannels, 22);
  buffer.writeUInt32LE(sampleRate, 24);
  buffer.writeUInt32LE(byteRate, 28);
  buffer.writeUInt16LE(blockAlign, 32);
  buffer.writeUInt16LE(16, 34); // Bits per sample
  buffer.write("data", 36);
  buffer.writeUInt32LE(dataSize, 40);

  for (let i = 0; i < numSamples; i++) {
    const t = i / sampleRate;
    const attack = Math.min(1.0, t / 0.05);
    const decay = Math.exp(-t * 2.0);
    const envelope = attack * decay;

    const f1 = 432;
    const f2 = 864;
    const s1 = Math.sin(2 * Math.PI * f1 * t);
    const s2 = 0.3 * Math.sin(2 * Math.PI * f2 * t);
    const val = (s1 + s2) * envelope * 0.45;

    const intSample = Math.max(-32767, Math.min(32767, Math.floor(val * 32767)));
    buffer.writeInt16LE(intSample, 44 + i * 2);
  }

  const audioDir = path.join(__dirname, "src", "overlay", "audio");
  if (!fs.existsSync(audioDir)) fs.mkdirSync(audioDir, { recursive: true });
  fs.writeFileSync(path.join(audioDir, "chime.wav"), buffer);
  console.log("Created chime.wav:", buffer.length, "bytes");
}

generateChime();
