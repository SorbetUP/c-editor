#!/usr/bin/env python3
import argparse, json, math, sys
from pathlib import Path
from PIL import Image, ImageChops, ImageDraw, ImageEnhance, ImageFilter, ImageStat

INFRA_ERROR, PARITY_FAIL = 3, 2

def pixels(im): return list(im.getdata())

def health(im):
    sample=im.convert('RGB').copy(); sample.thumbnail((320,240),Image.Resampling.BOX)
    colors=sample.getcolors(maxcolors=320*240); gray=sample.convert('L'); hist=gray.histogram(); n=sum(hist) or 1
    return {'uniqueColors':len(colors) if colors is not None else 320*240,
            'luminanceStdDev':ImageStat.Stat(gray).stddev[0],
            'luminanceEntropy':-sum((v/n)*math.log2(v/n) for v in hist if v)}

def sane(h,t): return h['uniqueColors']>=t.get('minUniqueColors',8) and h['luminanceStdDev']>=t.get('minLuminanceStdDev',1) and h['luminanceEntropy']>=t.get('minLuminanceEntropy',.5)

def ssim(a,b,block=16):
    a,b=a.convert('L'),b.convert('L'); out=[]; c1=(.01*255)**2; c2=(.03*255)**2
    for y in range(0,a.height,block):
      for x in range(0,a.width,block):
        box=(x,y,min(x+block,a.width),min(y+block,a.height)); av=pixels(a.crop(box)); bv=pixels(b.crop(box)); n=len(av)
        if not n: continue
        ma=sum(av)/n; mb=sum(bv)/n; va=sum((v-ma)**2 for v in av)/n; vb=sum((v-mb)**2 for v in bv)/n
        cov=sum((av[i]-ma)*(bv[i]-mb) for i in range(n))/n; den=(ma*ma+mb*mb+c1)*(va+vb+c2)
        out.append(((2*ma*mb+c1)*(2*cov+c2))/den if den else 1)
    out.sort(); return {'mean':sum(out)/len(out) if out else 1,'p05':out[max(0,math.ceil(len(out)*.05)-1)] if out else 1,'min':out[0] if out else 1}

def overlap(a,b,dx,dy):
    ax=max(0,dx); ay=max(0,dy); bx=max(0,-dx); by=max(0,-dy); w=min(a.width-ax,b.width-bx); h=min(a.height-ay,b.height-by)
    if w<=0 or h<=0:return 1
    d=ImageChops.difference(a.crop((ax,ay,ax+w,ay+h)),b.crop((bx,by,bx+w,by+h))); v=pixels(d)
    return sum(v)/(len(v)*255) if v else 0

def shift_hint(a,b,radius=4):
    size=(max(1,a.width//2),max(1,a.height//2)); a=a.convert('L').filter(ImageFilter.GaussianBlur(1.2)).resize(size,Image.Resampling.BOX); b=b.convert('L').filter(ImageFilter.GaussianBlur(1.2)).resize(size,Image.Resampling.BOX)
    r=max(1,math.ceil(radius/2)); base=overlap(a,b,0,0); best=(base,0,0)
    for y in range(-r,r+1):
      for x in range(-r,r+1): best=min(best,(overlap(a,b,x,y),x*2,y*2))
    improvement=(base-best[0])/base if base>1e-12 else 0
    return {'bestDx':best[1],'bestDy':best[2],'unshiftedNmae':base,'bestNmae':best[0],'relativeImprovement':improvement,'suspectedGlobalShift':(abs(best[1])>=2 or abs(best[2])>=2) and improvement>=.12}

def tile_hotspots(delta,mask,tile=48):
    out=[]
    for y in range(0,delta.height,tile):
      for x in range(0,delta.width,tile):
        box=(x,y,min(x+tile,delta.width),min(y+tile,delta.height)); mv=pixels(mask.crop(box)); dv=pixels(delta.crop(box)); n=len(mv) or 1
        ratio=sum(bool(v) for v in mv)/n; mae=sum(sum(p) for p in dv)/(n*3) if dv else 0
        if ratio or mae: out.append({'x':x,'y':y,'width':box[2]-x,'height':box[3]-y,'significantPixelRatio':ratio,'meanAbsoluteDifference':mae})
    return sorted(out,key=lambda q:(q['significantPixelRatio'],q['meanAbsoluteDifference']),reverse=True)[:8]

def edge_ratio(a,b,threshold=20):
    def m(im): return im.convert('L').filter(ImageFilter.GaussianBlur(1)).filter(ImageFilter.FIND_EDGES).point(lambda v:255 if v>=threshold else 0,mode='1')
    ea,eb=m(a),m(b); xor=ImageChops.logical_xor(ea,eb); union=ImageChops.logical_or(ea,eb); xc=sum(bool(v) for v in pixels(xor)); uc=sum(bool(v) for v in pixels(union))
    return xc/uc if uc else 0

def coarse(a,b,factor=4):
    size=(max(1,a.width//factor),max(1,a.height//factor)); a=a.resize(size,Image.Resampling.LANCZOS); b=b.resize(size,Image.Resampling.LANCZOS); d=ImageChops.difference(a,b); v=pixels(d); n=len(v) or 1
    return {'normalizedMeanAbsoluteError':sum(sum(p) for p in v)/(n*3*255),'blockSsim':ssim(a,b,max(4,16//factor))['mean']}

def review(a,b,d,mask,path):
    half=(max(1,a.width//2),max(1,a.height//2)); aa=a.resize(half); bb=b.resize(half); dd=ImageEnhance.Contrast(d).enhance(4).resize(half)
    ov=b.convert('RGBA'); red=Image.new('RGBA',b.size,(255,0,0,0)); red.putalpha(mask.convert('L').point(lambda v:150 if v else 0)); ov=Image.alpha_composite(ov,red).convert('RGB').resize(half)
    sheet=Image.new('RGB',(half[0]*2,half[1]*2),'white'); sheet.paste(aa,(0,0));sheet.paste(bb,(half[0],0));sheet.paste(dd,(0,half[1]));sheet.paste(ov,(half[0],half[1])); draw=ImageDraw.Draw(sheet)
    for txt,xy in [('TAURI',(8,8)),('FREYA',(half[0]+8,8)),('DIFF x4',(8,half[1]+8)),('SIGNIFICANT',(half[0]+8,half[1]+8))]: draw.rectangle((xy[0]-3,xy[1]-3,xy[0]+120,xy[1]+15),fill='white');draw.text(xy,txt,fill='black')
    path.parent.mkdir(parents=True,exist_ok=True);sheet.save(path,optimize=True)

def compare_one(rp,cp,dp,vp,t):
    try:a=Image.open(rp).convert('RGB');b=Image.open(cp).convert('RGB');a.load();b.load()
    except Exception as e:return {'status':'INFRA_ERROR','error':f'PNG decode failed: {e}'}
    if a.size!=b.size:return {'status':'INFRA_ERROR','error':'dimension mismatch','referenceDimensions':list(a.size),'candidateDimensions':list(b.size)}
    ha,hb=health(a),health(b)
    if not sane(ha,t) or not sane(hb,t):return {'status':'INFRA_ERROR','error':'capture blank or degenerate','imageHealth':{'reference':ha,'candidate':hb}}
    d=ImageChops.difference(a,b); v=pixels(d); n=len(v) or 1; channel=t['channelDelta']; diff=sum(p!=(0,0,0) for p in v); sig=sum(max(p)>channel for p in v); mae=sum(sum(p) for p in v)/(n*3); rmse=math.sqrt(sum(sum(c*c for c in p) for p in v)/(n*3))/255
    mask=Image.new('1',a.size); mask.putdata([255 if max(p)>channel else 0 for p in v]); spots=tile_hotspots(d,mask,t.get('tileSize',48)); s=ssim(a,b,t.get('ssimBlockSize',16)); c=coarse(a,b,t.get('coarseScale',4)); sr=sig/n; nm=mae/255; mt=spots[0]['significantPixelRatio'] if spots else 0
    reasons=[]
    def gate(bad,metric,value,limit):
      if bad:reasons.append({'metric':metric,'value':value,'limit':limit})
    gate(sr>t['maxSignificantPixelRatio'],'significantPixelRatio',sr,t['maxSignificantPixelRatio']); gate(nm>t['maxNormalizedMeanAbsoluteError'],'normalizedMeanAbsoluteError',nm,t['maxNormalizedMeanAbsoluteError']); gate(s['mean']<t['minBlockSsim'],'blockSsim.mean',s['mean'],t['minBlockSsim']); gate(mt>t.get('maxTileSignificantPixelRatio',1),'maxTileSignificantPixelRatio',mt,t.get('maxTileSignificantPixelRatio',1)); gate(c['normalizedMeanAbsoluteError']>t.get('maxCoarseNmae',1),'coarseNmae',c['normalizedMeanAbsoluteError'],t.get('maxCoarseNmae',1))
    dp.parent.mkdir(parents=True,exist_ok=True);ImageEnhance.Contrast(d).enhance(4).save(dp,optimize=True);review(a,b,d,mask,vp)
    return {'status':'FAIL' if reasons else 'PASS','dimensions':list(a.size),'imageHealth':{'reference':ha,'candidate':hb},'differentPixels':diff,'differentPixelRatio':diff/n,'significantPixels':sig,'significantPixelRatio':sr,'significantDifferenceBoundingBox':list(mask.getbbox()) if mask.getbbox() else None,'meanAbsoluteDifference':mae,'normalizedMeanAbsoluteError':nm,'normalizedRootMeanSquareError':rmse,'blockSsim':s,'maxTileSignificantPixelRatio':mt,'hotspots':spots,'edgeStructureDisagreement':edge_ratio(a,b,t.get('edgeThreshold',20)),'coarseScale':c,'translationDiagnostic':shift_hint(a,b,t.get('translationRadius',4)),'gateReasons':reasons,'diff':str(dp),'review':str(vp)}

def stability(p,s,t):
    if not s.is_file():return {'status':'NOT_CAPTURED'}
    try:a=Image.open(p).convert('RGB');b=Image.open(s).convert('RGB')
    except Exception as e:return {'status':'INFRA_ERROR','error':str(e)}
    if a.size!=b.size:return {'status':'INFRA_ERROR','error':'dimension mismatch'}
    v=pixels(ImageChops.difference(a,b));n=len(v) or 1; ratio=sum(max(q)>t['channelDelta'] for q in v)/n; nm=sum(sum(q) for q in v)/(n*3*255); bad=ratio>t.get('maxStabilitySignificantPixelRatio',.005) or nm>t.get('maxStabilityNmae',.002)
    return {'status':'UNSTABLE' if bad else 'STABLE','significantPixelRatio':ratio,'normalizedMeanAbsoluteError':nm}

def metadata(ref,cand):
    issues=[]
    try:r=json.loads((ref/'metadata.json').read_text());c=json.loads((cand/'staging.json').read_text())
    except Exception as e:return [f'metadata missing/invalid: {e}']
    if r.get('scenarioId')!=c.get('scenarioId'):issues.append('scenario mismatch')
    for k in ('width','height','scaleFactor','deviceScaleFactor'):
      if (r.get('viewport') or {}).get(k)!=(c.get('viewport') or {}).get(k):issues.append(f'viewport {k} mismatch')
    if (r.get('fixture') or {}).get('files')!=(c.get('fixture') or {}).get('files'):issues.append('fixture file/hash list mismatch')
    return issues

def run(ref,cand,cfg,out):
    t=cfg['thresholds']; issues=metadata(ref,cand); rows=[]
    for e in cfg['checkpoints']:
      cid=e['id'];rp=ref/cid/'static.png';cp=cand/cid/'static.png';row={'checkpoint':cid,'gate':e.get('gate',True)}
      if not rp.is_file() or not cp.is_file():row.update(status='INFRA_ERROR',error='missing capture')
      else:
        row.update(compare_one(rp,cp,out/'diffs'/f'{cid}.png',out/'review'/f'{cid}.png',t));row['stability']={'reference':stability(rp,ref/cid/'stability.png',t),'candidate':stability(cp,cand/cid/'stability.png',t)}
        if e.get('requireStability') and any(x['status']!='STABLE' for x in row['stability'].values()):row.update(status='INFRA_ERROR',error='required stable capture failed')
      rows.append(row)
    statuses=[r['status'] for r in rows];fails=[r for r in rows if r['status']=='FAIL' and r['gate']];overall='INFRA_ERROR' if issues or 'INFRA_ERROR' in statuses else ('FAIL' if fails else 'PASS')
    return {'schemaVersion':2,'status':overall,'thresholds':t,'metadataValidation':{'status':'PASS' if not issues else 'INFRA_ERROR','issues':issues},'summary':{'pass':statuses.count('PASS'),'fail':statuses.count('FAIL'),'infraError':statuses.count('INFRA_ERROR'),'total':len(rows),'gatedFailures':len(fails)},'checkpoints':rows}

def markdown(r):
    lines=['# Freya visual parity','',f"Overall: **{r['status']}**",'','| Checkpoint | Status | Significant | NMAE | SSIM | Worst tile | Shift |','|---|---:|---:|---:|---:|---:|---:|']
    for q in r.get('checkpoints',[]):
      if q['status']=='INFRA_ERROR':lines.append(f"| {q['checkpoint']} | INFRA_ERROR | - | - | - | - | - |")
      else:
        sh=q['translationDiagnostic'];hint=f"{sh['bestDx']:+d},{sh['bestDy']:+d}" if sh['suspectedGlobalShift'] else '-';lines.append(f"| {q['checkpoint']} | {q['status']} | {q['significantPixelRatio']:.3%} | {q['normalizedMeanAbsoluteError']:.5f} | {q['blockSsim']['mean']:.5f} | {q['maxTileSignificantPixelRatio']:.1%} | {hint} |")
    if r.get('metadataValidation',{}).get('issues'):lines+=['','Metadata INFRA_ERROR:']+[f"- {x}" for x in r['metadataValidation']['issues']]
    return '\n'.join(lines)+'\n'

def main():
    p=argparse.ArgumentParser();p.add_argument('--reference',type=Path,required=True);p.add_argument('--candidate',type=Path,required=True);p.add_argument('--config',type=Path,required=True);p.add_argument('--output',type=Path,required=True);p.add_argument('--hard-gate',action='store_true');a=p.parse_args();a.output.mkdir(parents=True,exist_ok=True)
    try:r=run(a.reference,a.candidate,json.loads(a.config.read_text()),a.output)
    except Exception as e:r={'schemaVersion':2,'status':'INFRA_ERROR','error':str(e),'summary':{'pass':0,'fail':0,'infraError':1,'total':0},'checkpoints':[]}
    (a.output/'report.json').write_text(json.dumps(r,indent=2)+'\n');(a.output/'summary.md').write_text(markdown(r));print(markdown(r),end='')
    return INFRA_ERROR if r['status']=='INFRA_ERROR' else (PARITY_FAIL if r['status']=='FAIL' and a.hard_gate else 0)
if __name__=='__main__':sys.exit(main())
