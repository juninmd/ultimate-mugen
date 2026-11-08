use crate::i18n::tr3;

const COUNT: usize = 8;

pub fn tip(n: usize) -> &'static str {
    match n % COUNT {
        0 => tr3("Dica: F11 alterna tela cheia · F3 mostra o FPS.", "Tip: F11 toggles fullscreen · F3 shows the FPS.", "Consejo: F11 alterna pantalla completa · F3 muestra los FPS."),
        1 => tr3("Dica: use Importar M.U.G.E.N no menu principal para adicionar um .zip de personagens.", "Tip: use Import M.U.G.E.N in the main menu to add a character .zip.", "Consejo: usa Importar M.U.G.E.N en el menú principal para añadir un .zip de personajes."),
        2 => tr3("Dica: no Treino, F2 exibe as hitboxes.", "Tip: in Training, F2 shows the hitboxes.", "Consejo: en Entrenamiento, F2 muestra las hitboxes."),
        3 => tr3("Dica: ESPECIAL e SUPER executam golpes com um único toque.", "Tip: SPECIAL and SUPER run moves with a single tap.", "Consejo: ESPECIAL y SUPER ejecutan golpes con un solo toque."),
        4 => tr3("Dica: em Vídeo, a Qualidade da arte aumenta a nitidez em telas 4K.", "Tip: in Video, Art quality sharpens the picture on 4K screens.", "Consejo: en Vídeo, la Calidad del arte mejora la nitidez en pantallas 4K."),
        5 => tr3("Dica: PageUp/PageDown e a roda do mouse rolam a lista de lutadores.", "Tip: PageUp/PageDown and the mouse wheel scroll the fighter list.", "Consejo: AvPág/RePág y la rueda del ratón desplazan la lista de luchadores."),
        6 => tr3("Dica: DualShock/DualSense vibram em golpes, defesa e K.O.", "Tip: DualShock/DualSense rumble on hits, blocks and K.O.", "Consejo: DualShock/DualSense vibran en golpes, bloqueos y K.O."),
        _ => tr3("Dica: Tab filtra a seleção entre todos, originais e M.U.G.E.N.", "Tip: Tab filters the select screen between all, originals and M.U.G.E.N.", "Consejo: Tab filtra la selección entre todos, originales y M.U.G.E.N."),
    }
}
