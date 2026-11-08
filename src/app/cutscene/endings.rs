use super::story::clean_id;
use super::{pg, Page};

pub fn ending_script(hero_id: &str, hero: &str, hero_quote: &str) -> Vec<Page> {
    let id = clean_id(hero_id);
    let first = hero.split_whitespace().next().unwrap_or(hero);
    let (boss_fall, reply, epilogue_1, epilogue_2) = match id {
        "kaito" => (
            "Impossível... mesmo o meu Eclipse Absoluto foi despedaçado pelo seu punho...",
            "Você enxergou todos os futuros possíveis, Oboro — menos aquele em que eu me recusava a cair.",
            "Com o último impacto do Punho do Dragão, a coroa negra do eclipse se desfaz em pétalas de luz solar.",
            "Kaito retorna ao templo nas montanhas ao amanhecer, carregando não um título, mas a chama de um novo começo.",
        ),
        "shion" => (
            "Meus olhos previram mil sombras... mas não a lâmina que cortou o próprio vazio...",
            "Uma verdadeira sombra não existe no futuro que você vê. Ela nasce no instante em que sua guarda falha.",
            "O salão ancestral silencia enquanto Shion embainha sua adaga e o véu escarlate se dissipa no vento.",
            "Livre da maldição que prendia seu clã ao Eclipse, Shion desaparece na brisa da aurora como uma lenda silenciosa.",
        ),
        "akio" => (
            "Nem mesmo a distorção do espaço-tempo conseguiu alcançar a sua velocidade...",
            "O destino pode ser rápido, Oboro, mas o meu relâmpago chega antes mesmo do som do trovão!",
            "Um clarão dourado rasga as nuvens negras, devolvendo o azul elétrico ao céu sobre a arena devastada.",
            "Akio ergue o punho faiscante no topo das ruínas, provando que nenhum deus pode acorrentar a tempestade.",
        ),
        "suzume" => (
            "Essa luz... ela queima até mesmo as chamas negras da minha eternidade...",
            "Descanse, Oboro. A noite terminou, e os espíritos que você aprisionou finalmente estão livres.",
            "As plumas celestes de Suzume dançam pelo céu, transformando as cinzas do Eclipse em chuva cristalina.",
            "No santuário reconstruído, os sinos tocam pela primeira vez sob um sol puro, saudando sua guardiã sagrada.",
        ),
        "bruno" => (
            "Tanta técnica divina... derrotada pela pura obstinação de um lutador mortal...",
            "Eu avisei, figurão! No ringue de verdade, coração e punho fechado valem mais que qualquer profecia!",
            "O altar de pedra desmorona enquanto Bruno ergue o cinturão invisível sob os primeiros raios da manhã.",
            "De volta às ruas, o campeão celebra com o povo: o homem que nocauteou o próprio Eclipse no último assalto.",
        ),
        "sylene" => (
            "O calor do meu sol negro... congelado em silêncio absoluto...",
            "A eternidade que você buscava no fogo do eclipse sempre pertenceu ao meu gelo soberano.",
            "Cristais de gelo boreal envolvem o trono quebrado, refletindo a aurora em mil prismas cintilantes.",
            "Sylene caminha serenamente pelo salão congelado, soberana incontestável do inverno e da nova alvorada.",
        ),
        "volta" => (
            "Toda a gravidade do abismo... rompida por uma descarga humana...",
            "Fim de linha para o seu sistema, Oboro! O futuro não é código fechado — a gente reescreve na marra!",
            "Arcos voltaicos azuis percorrem o horizonte quando as torres de Neo-Tóquio voltam a brilhar ao longe.",
            "Volta ajusta os visores com um sorriso elétrico e parte acelerando rumo a uma cidade livre da escuridão.",
        ),
        "lian" => (
            "Minha força absoluta... desviada e devolvida contra mim mesmo...",
            "A rocha rígida se parte sob a própria arrogância; a água vence porque nunca luta contra a sua natureza.",
            "Uma brisa suave atravessa o campo de batalha, levando consigo a névoa opressora do Eclipse Escarlate.",
            "De volta ao vale dos bambus, Lian serve o chá da manhã em paz — a harmonia do mundo foi restaurada.",
        ),
        "garan" => (
            "Nem o peso de um astro em colapso conseguiu dobrar os seus joelhos...",
            "Você tentou esmagar uma montanha com ilusões, Oboro! Nada supera o aço forjado no suor e na batalha!",
            "A gargalhada triunfante de Garan ecoa pelas crateras do vulcão enquanto o céu recupera seu brilho dourado.",
            "Em sua forja ancestral, o colosso ergue o martelo mais uma vez, agora celebrado como o Titã que quebrou o Eclipse.",
        ),
        "marlow" => (
            "Impossível... todas as probabilidades apontavam para a minha vitória...",
            "Probabilidade não ganha mesa, parceiro. Você mostrou suas cartas cedo demais — e eu tinha o ás na manga.",
            "Marlow acende um sorriso irônico, lança uma carta dourada sobre o trono partido e ajeita o chapéu.",
            "Nos portos iluminados pelo sol nascente, ninguém ousa duvidar do apostador que levou a banca do próprio destino.",
        ),
        "kaname" => (
            "Em que instante... a sua espada atravessou a minha dimensão...?",
            "Antes mesmo de a sua forma final respirar. O iaijutsu perfeito corta a intenção antes do movimento.",
            "Com um clique metálico preciso, Kaname guarda a lâmina na bainha e a lua carmesim se divide ao meio.",
            "O espadachim segue solitário pela estrada banhada pelo sol, tendo alcançado o corte supremo que sua linhagem buscava.",
        ),
        "jin" => (
            "Minhas chamas do eclipse... devoradas por um fogo ainda mais feroz...",
            "Seu fogo nascia do vazio, Oboro. O meu queima com a fúria de quem nunca aceitou ser controlado!",
            "Chamas azul-celeste consomem os últimos vestígios da escuridão, limpando o ar pesado do santuário final.",
            "Jin observa o horizonte claro com as mãos nos bolsos, pronto para qualquer desafio que o novo mundo traga.",
        ),
        "daichi" => (
            "O próprio solo se ergueu para negar o meu julgamento final...",
            "Você esqueceu que este mundo tem raízes profundas. Enquanto eu proteger esta terra, ela jamais cairá!",
            "As fendas abertas pelo combate se fecham e brotos verdes surgem entre as rochas banhadas pela luz da manhã.",
            "Daichi retorna à sua vila recebido por aplausos, o guardião inabalável que sustentou o céu com os próprios ombros.",
        ),
        "valdric" => (
            "Meu poder divino... drenado por outro senhor da escuridão...",
            "Seu Eclipse não passava de um prelúdio, Oboro. A verdadeira noite eterna pertence somente à minha linhagem.",
            "Valdric absorve as últimas brasas carmesins em seu manto enquanto a lua cede lugar a um crepúsculo imperial.",
            "Do alto de seu castelo ancestral, o lorde imortal contempla um mundo onde nenhum rival resta acima de seu trono.",
        ),
        "haruki" => (
            "Eu vi o fim dos tempos... como pude perder para alguém que apenas sorri e avança...?",
            "Porque você luta preso ao que já aconteceu! Eu invento o meu próximo golpe no meio do salto!",
            "Haruki faz um sinal de vitória para o céu que se abre em tons vibrantes de laranja e dourado.",
            "De volta às ruas movimentadas, o jovem prodígio já procura o próximo adversário — a aventura está só começando!",
        ),
        _ => (
            "Impossível... nem mesmo o meu Eclipse Absoluto pôde deter você...",
            if hero_quote.is_empty() {
                "Você previu cada golpe meu, mas jamais entenderia a força de quem luta pelo próprio amanhã!"
            } else {
                hero_quote
            },
            "A coroa negra do Eclipse se desfaz em mil fragmentos de luz diante do golpe decisivo.",
            "Quando o sol finalmente nasce sobre a arena silenciosa, o mundo inteiro celebra o triunfo de uma nova lenda.",
        ),
    };
    vec![
        pg(2, boss_fall),
        pg(1, reply),
        pg(0, epilogue_1),
        pg(0, epilogue_2),
        pg(
            0,
            format!(
                "FIM DA JORNADA DE {} — O ECLIPSE FOI SUPERADO.",
                first.to_uppercase()
            ),
        ),
    ]
}
