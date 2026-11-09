use super::{pg, Page};
use super::story::clean_id;

pub fn climax_script(hero_id: &str, hero: &str, boss: &str) -> Vec<Page> {
    let id = clean_id(hero_id);
    let hero_shout = match id {
        "kaito" => "Não há abismo que apague o fogo do meu espírito! Eu vou quebrar o seu trono!",
        "shion" => "Suas ilusões já não enganam meus olhos. Sua linha vital está exposta!",
        "akio" => "Sinta a fúria do relâmpago que corta os céus! Nada pode conter esta tempestade!",
        "suzume" => "Pelas asas sagradas da alvorada, eu purificarei até o último resquício da sua escuridão!",
        "bruno" => "Guarda alta e queixo no peito! Esse é o último assalto da sua tirania!",
        "sylene" => "Seu tempo acabou, soberano da noite. O gelo eterno reclama seu último suspiro!",
        "volta" => "Geradores na sobrecarga máxima! Vou descarregar um gigawatt nessa sua soberba!",
        "lian" => "A torrente do destino flui para a frente. O seu império de sombras se desfaz agora!",
        "garan" => "A terra racha sob o meu poder! Você não vai aguentar o peso da minha marreta!",
        "marlow" => "Todas as suas cartas foram reveladas, Oboro! Esta é a mão que decreta o seu fim!",
        "kaname" => "A técnica suprema herdada por gerações... corte de um milhão de luas!",
        "jin" => "Queime até não sobrar nem poeira! Nossas chamas devorarão a sua eternidade!",
        "daichi" => "Esta montanha permanece inabalável! Seu eclipse não tem força para nos derrubar!",
        "valdric" => "Você ousou reivindicar a noite? Curve-se diante do verdadeiro pesadelo carmesim!",
        "haruki" => "Seja o que for que você invoque, meu combo definitivo vai passar por cima!",
        _ => "Chegou a sua hora! Não recuarei nem diante de um deus!",
    };
    vec![
        pg(0, format!("Golpes supersônicos rasgam a atmosfera. {hero} e {boss} duelam em velocidade frenética entre fagulhas de pura energia cósmica.")),
        pg(1, hero_shout),
        pg(2, "Fascinante... você quebrou o Eclipse Absoluto e forçou meu espírito ao limite extremo!"),
        pg(0, "O vórtice cósmico estremece. Relâmpagos cianos e uma aura supernova irrompem do manto rasgado de Mikazuki Oboro."),
        pg(2, "Rompa-se, última corrente do infinito! Eis a gênese primordial — ECLIPSE PRIMORDIAL!"),
        pg(1, "Venha com todo o seu poder cósmico! Vamos decidir o destino deste mundo agora!"),
    ]
}

pub fn defeat_script(hero_id: &str, hero: &str, boss: &str) -> Vec<Page> {
    let id = clean_id(hero_id);
    let finish_quote = match id {
        "kaito" => "ESTE É O PODER DA VONTADE HUMANA! DESAPAREÇA NA ALVORADA!",
        "shion" => "A escuridão se dissipa perante o fio invisível da verdade.",
        "akio" => "O trovão final caiu! A noite eterna foi dizimada!",
        "suzume" => "Que as asas da luz celeste abram o caminho para um novo amanhecer!",
        "bruno" => "Nocaute definitivo! Pode soar o gongo, o campeão do povo venceu!",
        "sylene" => "Seu eclipse congelou para sempre no silêncio da eternidade.",
        "volta" => "Circuito finalizado! Apagão total nas suas ilusões cósmicas!",
        "lian" => "A tempestade findou. A água cristalina reflete novamente a paz.",
        "garan" => "Nem titãs nem deuses resistem à força da determinação!",
        "marlow" => "Fim de jogo, chefe. A casa sempre perde para quem aposta tudo.",
        "kaname" => "A lâmina guardada no silêncio... cortou a própria eternidade.",
        "jin" => "Suas cinzas foram dispersadas pelo vento. A noite ardeu por completo.",
        "daichi" => "A terra firme protegeu a todos. O sol finalmente voltou a brilhar.",
        "valdric" => "O eclipse pereceu... e a verdadeira lenda da noite perdura.",
        "haruki" => "Vitória limpa e incontestável! Essa luta entra para a história!",
        _ => "A sua era de tirania acabou! O eclipse finalmente encontrou o seu fim!",
    };
    vec![
        pg(0, format!("Em um estouro ofuscante de energia, {hero} avança em velocidade estelar e desfere o golpe decisivo no coração do Eclipse Primordial!")),
        pg(1, finish_quote),
        pg(2, "Como... como pôde uma chama mortal... despedaçar a eternidade do meu vazio...?"),
        pg(0, format!("A katana cósmica de {boss} se fragmenta em miríades de partículas celestes cintilantes.")),
        pg(0, "A lua negra que cobria o céu racha e se dissolve por completo. Os primeiros raios da alvorada dourada banham a terra."),
        pg(2, "O eclipse... terminou... O amanhã... pertence a vocês..."),
    ]
}
