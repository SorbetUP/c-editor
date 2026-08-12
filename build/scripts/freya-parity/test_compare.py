import tempfile
import unittest
from pathlib import Path
from PIL import Image, ImageDraw
from compare import compare_one

THRESHOLDS = {
    'channelDelta': 12,
    'maxSignificantPixelRatio': 0.005,
    'maxNormalizedMeanAbsoluteError': 0.004,
    'minBlockSsim': 0.985,
}

class ComparatorTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)

    def tearDown(self):
        self.temp.cleanup()

    def save(self, name, image):
        path = self.root / name
        image.save(path)
        return path

    def test_identical_passes(self):
        image = Image.new('RGB', (128, 96), 'white')
        a = self.save('a.png', image); b = self.save('b.png', image.copy())
        result = compare_one(a, b, self.root / 'diff.png', THRESHOLDS)
        self.assertEqual(result['status'], 'PASS')
        self.assertEqual(result['differentPixels'], 0)
        self.assertAlmostEqual(result['blockSsim'], 1.0)

    def test_small_raster_noise_is_tolerated(self):
        a = Image.new('RGB', (128, 96), (240, 240, 240))
        b = a.copy()
        draw = ImageDraw.Draw(b)
        for x in range(10, 110, 10):
            draw.point((x, 48), fill=(250, 250, 250))
        result = compare_one(self.save('a.png', a), self.save('b.png', b), self.root / 'diff.png', THRESHOLDS)
        self.assertEqual(result['status'], 'PASS')

    def test_three_pixel_layout_shift_fails(self):
        a = Image.new('RGB', (256, 128), 'white'); b = a.copy()
        ImageDraw.Draw(a).rectangle((20, 20, 220, 100), fill='black')
        ImageDraw.Draw(b).rectangle((23, 20, 223, 100), fill='black')
        result = compare_one(self.save('a.png', a), self.save('b.png', b), self.root / 'diff.png', THRESHOLDS)
        self.assertEqual(result['status'], 'FAIL')
        self.assertGreater(result['significantPixelRatio'], THRESHOLDS['maxSignificantPixelRatio'])

    def test_dimension_mismatch_is_infra_error(self):
        a = Image.new('RGB', (128, 96), 'white'); b = Image.new('RGB', (127, 96), 'white')
        result = compare_one(self.save('a.png', a), self.save('b.png', b), self.root / 'diff.png', THRESHOLDS)
        self.assertEqual(result['status'], 'INFRA_ERROR')

if __name__ == '__main__':
    unittest.main()
