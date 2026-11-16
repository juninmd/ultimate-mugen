from pathlib import Path
from PIL import Image
import hashlib,json
base=Path('assets/chars/sprites/garan')
out=Path('docs/evidence/native-identity-standardization/roster-family-audit/reviews/garan-airlowkick-family-pass-20261112')
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
frames=['jumpTop','akick','airLowKick','jumpDown','jumpDown']
source={n:{'path':f'assets/chars/sprites/garan/{n}.png','sha256':sha(base/f'{n}.png'),'bbox_alpha_gt_16':list((lambda im:im.getchannel('A').point(lambda x:255 if x>16 else 0).getbbox())(Image.open(base/f'{n}.png').convert('RGBA')))} for n in sorted(set(frames))}
pq=json.loads((out/'palette-seed-0'/'palette-qa.json').read_text(encoding='utf-8'))
report={'date':'2026-11-12','character':'garan','family':'airLowKick','status':'visual_static_pass_after_jumpDown_integration','sequence':[{'index':i,'sprite':name+'.png','sha256':sha(base/f'{name}.png')} for i,name in enumerate(frames)],'sources':source,'canonicalAnchor':{'path':'assets/chars/sprites/garan/04_special.png','sha256':sha(base/'04_special.png')},'visualReview':{'identity':'pass','pose_readability':'pass','scale_and_floor':'pass','pixel_art_consistency':'pass','unrequested_effects':'none','nativeEvidence':'contact-native.png','nearest2xEvidence':'contact-nearest-2x.png'},'paletteQA':{'status':pq['status'],'colors':48,'strict':True,'deltaEThreshold':0.13,'maxDeltaE':pq['checks'][-1]['value'],'report':'palette-seed-0/palette-qa.json'},'runtimeReview':'not_performed','testsRun':False,'supersedes':'The earlier correction_required family finding predated the integrated Garan jumpDown replacement; current ordered frames were reviewed after integration.','activeAssetsChanged':False}
(out/'family-review.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
