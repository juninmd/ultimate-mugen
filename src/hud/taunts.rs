use crate::i18n::{get, Lang};

const OPENING: [[&str; 5]; 3] = [
    [
        "Vamos ver do que você é capaz!",
        "Prepare-se!",
        "Não vou pegar leve!",
        "Que vença o melhor!",
        "Mostre sua força!",
    ],
    [
        "Let's see what you've got!",
        "Get ready!",
        "I won't go easy on you!",
        "May the best fighter win!",
        "Show me your strength!",
    ],
    [
        "¡Veamos de qué eres capaz!",
        "¡Prepárate!",
        "¡No me voy a contener!",
        "¡Que gane el mejor!",
        "¡Muestra tu fuerza!",
    ],
];

const ANSWER: [[&str; 5]; 3] = [
    [
        "Só isso?",
        "Você não me assusta.",
        "Venha com tudo!",
        "Hmpf... entediante.",
        "Vai se arrepender.",
    ],
    [
        "Is that all?",
        "You don't scare me.",
        "Bring it on!",
        "Hmpf... boring.",
        "You'll regret this.",
    ],
    [
        "¿Solo eso?",
        "No me asustas.",
        "¡Con todo!",
        "Hmpf... aburrido.",
        "Te vas a arrepentir.",
    ],
];

fn row(lang: Lang) -> usize {
    match lang {
        Lang::Pt => 0,
        Lang::En => 1,
        Lang::Es => 2,
    }
}

pub fn generic_line(answer: bool, pick: usize) -> &'static str {
    let table = if answer { &ANSWER } else { &OPENING };
    table[row(get())][pick % 5]
}
