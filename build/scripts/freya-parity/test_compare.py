import importlib.util
import json
import tempfile
import unittest
from pathlib import Path
from PIL import Image, ImageDraw

spec = importlib.util.spec_from_file_location('parity_compare', Path(__file__).with_name('compare.py'))
mod = importlib.util.module_from_spec(spec); spec.loader.exec_module(mod)

THRESHOLDS = {
    'channelDelta': 12, 'maxSignificantPixelRatio': 0.005,
    'maxNormalizedMeanAbsoluteError': 0.004, 'minBlockSsim': 0.985,
    'maxTileSignificantPixelRatio': 0.18, 'maxCoarseNmae': 0.012,
    'tileSize': 48, 'translationRadius': 4,
    'minUniqueColors': 8, 'minLuminanceStdDev': 1.0, 'minLuminanceEntropy': 0.5,
}

def patterned(size=(320, 240)):
    im = Image.new('RGB', size, '#f8f8f8'); d = ImageDraw.Draw(im)
    for x in range(0, size[0], 20): d.line((x, 0, x, size[1]), fill=(220, 220, 220))
    for y in range(0, size[1], 20): d.line((0, y, size[0], y), fill=(225, 225, 225))
    d.rectangle((25, 25, 105, 80), fill=(50, 80, 120)); d.rectangle((130, 30, 290, 52), fill=(30, 30, 30)); d.ellipse((210, 120, 260, 170), fill=(180, 70, 80))
    for i in range(8): d.rectangle((10+i*12, 190, 18+i*12, 198), fill=(20+i*25, 40+i*10, 210-i*20))
    return im

class CompareTests(unittest.TestCase):
    def compare(self, a, b):
        with tempfile.TemporaryDirectory() as td:
            root=Path(td); ap=root/'a.png'; bp=root/'b.png'; a.save(ap); b.save(bp)
            return mod.compare_one(ap,bp,root/'diff.png',root/'review.png',THRESHOLDS)

    def test_identical_passes_and_emits_diagnostics(self):
        result=self.compare(patterned(),patterned()); self.assertEqual(result['status'],'PASS'); self.assertEqual(result['significantPixelRatio'],0); self.assertFalse(result['translationDiagnostic']['suspectedGlobalShift']); self.assertEqual(result['blockSsim']['mean'],1.0)

    def test_small_antialias_like_noise_can_pass(self):
        a=patterned(); b=a.copy(); px=b.load()
        for y in range(100,103):
            for x in range(100,103): r,g,bb=px[x,y]; px[x,y]=(min(255,r+6),g,bb)
        result=self.compare(a,b); self.assertEqual(result['status'],'PASS'); self.assertGreater(result['differentPixelRatio'],0); self.assertEqual(result['significantPixelRatio'],0)

    def test_three_pixel_global_shift_fails_and_is_diagnosed(self):
        a=patterned(); b=Image.new('RGB',a.size,'white'); b.paste(a,(3,0)); result=self.compare(a,b)
        self.assertEqual(result['status'],'FAIL'); self.assertTrue(result['gateReasons']); self.assertGreaterEqual(abs(result['translationDiagnostic']['bestDx']),2); self.assertTrue(result['translationDiagnostic']['suspectedGlobalShift'])

    def test_local_missing_button_fails_even_when_global_ratio_is_small(self):
        a=patterned((640,480)); b=a.copy(); ImageDraw.Draw(b).rectangle((500,400,523,423),fill=(10,10,10)); result=self.compare(a,b)
        self.assertEqual(result['status'],'FAIL'); self.assertGreater(result['maxTileSignificantPixelRatio'],THRESHOLDS['maxTileSignificantPixelRatio']); self.assertIsNotNone(result['significantDifferenceBoundingBox'])

    def test_dimension_mismatch_is_infra_error(self): self.assertEqual(self.compare(patterned((320,240)),patterned((321,240)))['status'],'INFRA_ERROR')
    def test_blank_capture_is_infra_error(self):
        a=Image.new('RGB',(320,240),'white'); self.assertEqual(self.compare(a,a.copy())['status'],'INFRA_ERROR')

    def test_metadata_fixture_mismatch_is_infra_error(self):
        with tempfile.TemporaryDirectory() as td:
            root=Path(td); ref=root/'ref'; cand=root/'cand'; out=root/'out'
            for base in (ref,cand): (base/'cp').mkdir(parents=True); patterned().save(base/'cp'/'static.png')
            (ref/'metadata.json').write_text(json.dumps({'scenarioId':'x','viewport':{'width':320,'height':240,'scaleFactor':1,'deviceScaleFactor':1},'fixture':{'files':[{'path':'A','sha256':'1'}]}}))
            (cand/'staging.json').write_text(json.dumps({'scenarioId':'x','viewport':{'width':320,'height':240,'scaleFactor':1,'deviceScaleFactor':1},'fixture':{'files':[{'path':'A','sha256':'2'}]}}))
            report=mod.run(ref,cand,{'thresholds':THRESHOLDS,'checkpoints':[{'id':'cp'}]},out); self.assertEqual(report['status'],'INFRA_ERROR'); self.assertIn('fixture file/hash list mismatch',report['metadataValidation']['issues'])

if __name__ == '__main__': unittest.main()
