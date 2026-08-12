#!/usr/bin/env python3
import argparse
import json
import sys
from pathlib import Path
from PIL import Image, ImageChops, ImageEnhance

INFRA_ERROR = 3
PARITY_FAIL = 2


def block_ssim(a: Image.Image, b: Image.Image, block=16):
    a = a.convert('L')
    b = b.convert('L')
    width, height = a.size
    scores = []
    c1 = (0.01 * 255) ** 2
    c2 = (0.03 * 255) ** 2
    for y in range(0, height, block):
        for x in range(0, width, block):
            box = (x, y, min(x + block, width), min(y + block, height))
            av = list(a.crop(box).getdata())
            bv = list(b.crop(box).getdata())
            n = len(av)
            if not n:
                continue
            ma = sum(av) / n
            mb = sum(bv) / n
            va = sum((v - ma) ** 2 for v in av) / n
            vb = sum((v - mb) ** 2 for v in bv) / n
            cov = sum((av[i] - ma) * (bv[i] - mb) for i in range(n)) / n
            numerator = (2 * ma * mb + c1) * (2 * cov + c2)
            denominator = (ma * ma + mb * mb + c1) * (va + vb + c2)
            scores.append(numerator / denominator if denominator else 1.0)
    return sum(scores) / len(scores) if scores else 1.0


def compare_one(reference_path, candidate_path, diff_path, thresholds):
    try:
        reference = Image.open(reference_path).convert('RGB')
        candidate = Image.open(candidate_path).convert('RGB')
        reference.load(); candidate.load()
    except Exception as error:
        return {'status': 'INFRA_ERROR', 'error': f'PNG decode failed: {error}'}
    if reference.size != candidate.size:
        return {
            'status': 'INFRA_ERROR',
            'error': 'dimension mismatch',
            'referenceDimensions': list(reference.size),
            'candidateDimensions': list(candidate.size),
        }
    delta = ImageChops.difference(reference, candidate)
    pixels = list(delta.getdata())
    total = len(pixels)
    different = sum(1 for px in pixels if px != (0, 0, 0))
    significant = sum(1 for px in pixels if max(px) > thresholds['channelDelta'])
    absolute_sum = sum(sum(px) for px in pixels)
    mean_absolute = absolute_sum / (total * 3) if total else 0.0
    normalized_mae = mean_absolute / 255.0
    ssim = block_ssim(reference, candidate)
    significant_ratio = significant / total if total else 0.0
    different_ratio = different / total if total else 0.0
    max_delta = max((max(px) for px in pixels), default=0)
    passed = (
        significant_ratio <= thresholds['maxSignificantPixelRatio']
        and normalized_mae <= thresholds['maxNormalizedMeanAbsoluteError']
        and ssim >= thresholds['minBlockSsim']
    )
    diff_path.parent.mkdir(parents=True, exist_ok=True)
    ImageEnhance.Contrast(delta).enhance(4.0).save(diff_path)
    return {
        'status': 'PASS' if passed else 'FAIL',
        'dimensions': list(reference.size),
        'differentPixels': different,
        'differentPixelRatio': different_ratio,
        'significantPixels': significant,
        'significantPixelRatio': significant_ratio,
        'meanAbsoluteDifference': mean_absolute,
        'normalizedMeanAbsoluteError': normalized_mae,
        'blockSsim': ssim,
        'maxChannelDelta': max_delta,
        'diff': str(diff_path),
    }


def run(reference_dir, candidate_dir, config, output_dir):
    thresholds = config['thresholds']
    checkpoints = []
    for entry in config['checkpoints']:
        checkpoint = entry['id']
        reference = reference_dir / checkpoint / 'static.png'
        candidate = candidate_dir / checkpoint / 'static.png'
        record = {
            'checkpoint': checkpoint,
            'reference': str(reference),
            'candidate': str(candidate),
        }
        missing = [str(p) for p in (reference, candidate) if not p.is_file()]
        if missing:
            record.update(status='INFRA_ERROR', error=f'missing capture(s): {missing}')
        else:
            record.update(compare_one(reference, candidate, output_dir / 'diffs' / f'{checkpoint}.png', thresholds))
        checkpoints.append(record)
    statuses = [item['status'] for item in checkpoints]
    overall = 'INFRA_ERROR' if 'INFRA_ERROR' in statuses else ('FAIL' if 'FAIL' in statuses else 'PASS')
    return {
        'schemaVersion': 1,
        'status': overall,
        'thresholds': thresholds,
        'summary': {
            'pass': statuses.count('PASS'),
            'fail': statuses.count('FAIL'),
            'infraError': statuses.count('INFRA_ERROR'),
            'total': len(statuses),
        },
        'checkpoints': checkpoints,
    }


def markdown(report):
    lines = [
        '# Freya visual parity', '',
        f"Overall: **{report['status']}**", '',
        '| Checkpoint | Status | Changed px | Significant px | NMAE | block SSIM |',
        '|---|---:|---:|---:|---:|---:|',
    ]
    for item in report['checkpoints']:
        if item['status'] == 'INFRA_ERROR':
            lines.append(f"| {item['checkpoint']} | INFRA_ERROR | - | - | - | - |")
        else:
            lines.append(
                f"| {item['checkpoint']} | {item['status']} | {item['differentPixelRatio']:.4%} | "
                f"{item['significantPixelRatio']:.4%} | {item['normalizedMeanAbsoluteError']:.6f} | {item['blockSsim']:.6f} |"
            )
    return '\n'.join(lines) + '\n'


def main(argv=None):
    parser = argparse.ArgumentParser()
    parser.add_argument('--reference', required=True, type=Path)
    parser.add_argument('--candidate', required=True, type=Path)
    parser.add_argument('--config', required=True, type=Path)
    parser.add_argument('--output', required=True, type=Path)
    parser.add_argument('--hard-gate', action='store_true')
    args = parser.parse_args(argv)
    args.output.mkdir(parents=True, exist_ok=True)
    try:
        config = json.loads(args.config.read_text())
        report = run(args.reference, args.candidate, config, args.output)
    except Exception as error:
        report = {'schemaVersion': 1, 'status': 'INFRA_ERROR', 'summary': {'pass': 0, 'fail': 0, 'infraError': 1, 'total': 0}, 'error': str(error), 'checkpoints': []}
    (args.output / 'report.json').write_text(json.dumps(report, indent=2) + '\n')
    (args.output / 'summary.md').write_text(markdown(report))
    print(markdown(report), end='')
    if report['status'] == 'INFRA_ERROR':
        return INFRA_ERROR
    if report['status'] == 'FAIL' and args.hard_gate:
        return PARITY_FAIL
    return 0

if __name__ == '__main__':
    sys.exit(main())
