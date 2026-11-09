use std::io::Write;
fn main(){
    let p=std::env::args().nth(1).unwrap();
    let b=std::fs::read(p).unwrap();
    let s=ume_formats::sff::decode_sff(&b).unwrap();
    let mut o=std::io::stdout().lock();
    for sp in &s.sprites{
        let (ph,pp)=match &sp.palette{Some(p)=>{let mut h:u64=1469598103934665603;for v in p.iter(){h=(h^*v as u64).wrapping_mul(1099511628211)};(h,p[4..8].to_vec())},None=>(0,vec![])};
        let mut h:u64=1469598103934665603;
        if let Some(ix)=&sp.indexed{for v in ix.iter(){h=(h^*v as u64).wrapping_mul(1099511628211)}}
        writeln!(o,"{} {} {} {} {:x} {:x} {:?}",sp.group,sp.index,sp.width,sp.height,h,ph,pp).unwrap();
    }
}
