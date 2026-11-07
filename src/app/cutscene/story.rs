use super::{pg, Page};

pub fn clean_id(id: &str) -> &str {
    id.trim_start_matches("umf:")
        .rsplit('/')
        .next()
        .unwrap_or(id)
}

pub fn chapter_title(kind: u8, hero_name: &str) -> (&'static str, String) {
    match kind {
        0 => (
            "PRÓLOGO · CONVOCAÇÃO",
            format!("O CAMINHO DE {}", hero_name.to_uppercase()),
        ),
        1 => (
            "ATO FINAL · SALÃO DO ECLIPSE",
            "DIANTE DE MIKAZUKI OBORO".to_string(),
        ),
        3 => (
            "CLÍMAX · SELO QUEBRADO",
            "DESPERTAR: ECLIPSE ABSOLUTO".to_string(),
        ),
        _ => (
            "EPÍLOGO · ALVORADA",
            format!("A LENDA DE {}", hero_name.to_uppercase()),
        ),
    }
}

pub fn opening_script(hero_id: &str, hero: &str) -> Vec<Page> {
    let id = clean_id(hero_id);
    let (lore, vow) = match id {
        "kaito" => (
            "Nos dojos antigos, os pergaminhos proibidos arderam sob o brilho da lua rubra.",
            "Se o Eclipse ameaça o mundo, meu punho vai abrir o céu ao meio!",
        ),
        "shion" => (
            "As sombras do clã Kisaragi sussurram sobre um trono erguido além do tempo.",
            "Nenhuma ilusão sobrevive ao fio silencioso da minha lâmina.",
        ),
        "akio" => (
            "O trovão ancestral despertou nas montanhas quando o selo do Eclipse rachou.",
            "Vou cruzar essa tempestade na velocidade do relâmpago e derrubar esse falso deus!",
        ),
        "suzume" => (
            "Os espíritos do santuário celeste choram sob o véu carmesim que cobriu a noite.",
            "Minhas asas sagradas purificarão a escuridão até que o sol volte a nascer.",
        ),
        "bruno" => (
            "Nas arenas subterrâneas, campeões inteiros caíram diante dos emissários de Oboro.",
            "Previsão nenhuma aguenta um cruzado de direita bem dado no queixo!",
        ),
        "sylene" => (
            "Os espelhos astrais congelaram quando o olho de Mikazuki Oboro se abriu no abismo.",
            "Que ele veja o próprio fim refletido no gelo eterno da minha coroa.",
        ),
        "volta" => (
            "Os reatores de Neo-Tóquio entraram em sobrecarga com o pulso gravitacional do torneio.",
            "Hora de aumentar a voltagem ao máximo e fritar esse destino programado!",
        ),
        "lian" => (
            "As águas e os bambuzais do vale perderam a harmonia diante do chamado escarlate.",
            "Como a correnteza que molda a rocha, meu kung fu encontrará a brecha perfeita.",
        ),
        "garan" => (
            "A forja vulcânica tremeu: um poder antigo desafia a força bruta dos titãs.",
            "Podem vir deuses ou demônios — minha montanha de ferro não recua um passo!",
        ),
        "marlow" => (
            "Cartas marcadas pelo destino foram lançadas nas docas sob a névoa carmesim.",
            "Adoro quando a banca acha que já ganhou. É aí que eu viro o jogo.",
        ),
        "kaname" => (
            "A espada selada da família começou a ressoar com a presença de Mikazuki Oboro.",
            "Um único saque decidirá quem governa a noite sob esta lua sangrenta.",
        ),
        "jin" => (
            "As chamas azuis do submundo agitaram-se com o despertar do Soberano do Eclipse.",
            "Não existe trono alto demais para ser reduzido a cinzas pelo meu fogo.",
        ),
        "daichi" => (
            "A própria terra gemeu sob o peso das arenas erguidas pelo senhor das ilusões.",
            "Enquanto meus pés estiverem firmes no chão, ninguém derrubará o que eu protejo!",
        ),
        "valdric" => (
            "A sede ancestral do sangue nobre despertou diante da lua carmesim de Oboro.",
            "Só há espaço para um soberano da noite — e eu tomarei esse eclipse para mim.",
        ),
        "haruki" => (
            "O vento das ruas trouxe o desafio que todos os lutadores temiam aceitar.",
            "Dizem que o chefe final é invencível? Perfeito, eu precisava de um aquecimento!",
        ),
        _ => (
            "Lutadores de todas as eras foram atraídos pelo chamado da lua escarlate.",
            "Não importa quem aguarda no trono final. Eu chegarei até o fim!",
        ),
    };
    vec![
        pg(
            0,
            "O Torneio do Eclipse Escarlate convoca os maiores guerreiros sob uma lua de sangue.",
        ),
        pg(0, lore),
        pg(
            0,
            format!("Diante dos portões da arena, {hero} fecha os punhos e encara o horizonte."),
        ),
        pg(1, vow),
    ]
}

pub fn preboss_script(hero_id: &str, boss: &str, boss_line: &str) -> Vec<Page> {
    let id = clean_id(hero_id);
    let challenge = match id {
        "kaito" => "Suas ilusões não podem parar um espírito que queima além do limite!",
        "shion" => "Seus olhos veem o futuro, Oboro, mas não a sombra que já está atrás de você.",
        "akio" => "Vamos ver se seus olhos acompanham a velocidade do trovão divino!",
        "suzume" => "Em nome do santuário celeste, eu selarei essa lua corrompida aqui e agora!",
        "bruno" => "Menos papo furado sobre destino, campeão! Levanta a guarda e vem lutar!",
        "sylene" => "Seu trono de ilusões será seu próprio mausoléu de cristal, Oboro.",
        "volta" => "Seus cálculos têm um erro fatal: você não contava com um milhão de volts!",
        "lian" => "Uma mente presa ao futuro jamais enxerga o golpe que nasce no presente.",
        "garan" => "Eu vou esmagar esse seu destino com as minhas próprias mãos!",
        "marlow" => "Aposta alta, Oboro. Vamos ver quem está blefando quando a poeira baixar.",
        "kaname" => "Minha lâmina corta até mesmo os fios do destino que você julga controlar.",
        "jin" => "Sua lua vermelha vai parecer pálida quando minhas chamas consumirem este salão!",
        "daichi" => "Você olha de cima, mas esquece que toda torre cai quando a base treme!",
        "valdric" => "Um mortal brincando de deus da noite... vou lhe ensinar o verdadeiro terror.",
        "haruki" => "Você fala bonito, mas quero ver manter a pose depois do primeiro combo!",
        _ => "Então hoje você verá a sua primeira derrota com os próprios olhos!",
    };
    vec![
        pg(
            0,
            "Oito rivais tombaram. No topo do altar ancestral, a lua vermelha pulsa em silêncio.",
        ),
        pg(
            2,
            if boss_line.is_empty() {
                "Você chegou até aqui... exatamente como eu havia previsto."
            } else {
                boss_line
            },
        ),
        pg(
            2,
            format!(
                "Eu sou {boss}. Aquele que já contemplou mil finais para esta luta — e venceu em todos."
            ),
        ),
        pg(1, challenge),
        pg(
            2,
            "Que arrogância encantadora. Venha — tente quebrar o roteiro que escrevi para você.",
        ),
    ]
}

pub fn awakening_script(hero_id: &str, hero: &str) -> Vec<Page> {
    let id = clean_id(hero_id);
    let hero_reply = match id {
        "kaito" => "Pode despertar quantas formas quiser! Meu espírito também vai superar todo limite!",
        "shion" => "Então a máscara caiu. Mesmo o abismo inteiro não salvará você do meu golpe final.",
        "akio" => "Energia absurda... ótimo! Agora não preciso mais conter o meu trovão máximo!",
        "suzume" => "O selo se rompeu! Preciso purificar esse Eclipse Absoluto antes que consuma o mundo!",
        "bruno" => "Segundo round valendo tudo? É assim que uma verdadeira luta pelo título tem que ser!",
        "sylene" => "Que o próprio sol negro desça — meu zero absoluto congelará até a sua eternidade!",
        "volta" => "Leitura de energia fora da escala! Hora de ativar o sobrecarga total!",
        "lian" => "Quanto maior a tempestade, mais sereno é o centro do furacão. Venha, Oboro!",
        "garan" => "Hahaha! Agora sim! Mostre toda a sua força para que minha vitória seja absoluta!",
        "marlow" => "Dobrou a aposta depois de perder a primeira mão? Gostei do seu desespero!",
        "kaname" => "Uma presença avassaladora... mas ainda existe uma linha vital que minha espada alcança.",
        "jin" => "Finalmente parou de brincar! Vamos queimar até que só reste um de nós de pé!",
        "daichi" => "O chão está derretendo sob essa pressão... mas eu não vou dar um único passo para trás!",
        "valdric" => "Magnífico! Esse é o poder digno de ser devorado pelo verdadeiro lorde da noite!",
        "haruki" => "Transformação secreta de chefe final?! Agora a luta ficou lendária de verdade!",
        _ => "Mesmo que você transcenda a própria noite, eu vou derrotá-lo mais uma vez!",
    };
    vec![
        pg(
            0,
            format!(
                "Ajoelhado pela primeira vez em séculos, Mikazuki Oboro sorri enquanto o chão se despedaça diante de {hero}."
            ),
        ),
        pg(
            2,
            "Fascinante... você realmente forçou este corpo a sangrar e partiu o meu primeiro selo.",
        ),
        pg(
            0,
            "A lua vermelha no céu escurece por completo, cercada por uma coroa de fogo dourado e carmesim.",
        ),
        pg(
            2,
            "Contemple o que jamais mostrei a nenhum mortal: minha forma final — o ECLIPSE ABSOLUTO!",
        ),
        pg(1, hero_reply),
        pg(
            2,
            "Não haverá amanhã, nem piedade. Testemunhe o julgamento final do Eclipse!",
        ),
    ]
}
