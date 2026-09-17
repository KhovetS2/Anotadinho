//! Modelo e serialização estável de imagens inseridas no editor.

/// Alinhamento de apresentação de uma imagem.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ImageAlignment {
    /// Segue o fluxo normal do texto.
    #[default]
    Inline,
    /// Alinha à esquerda.
    Left,
    /// Centraliza.
    Center,
    /// Alinha à direita.
    Right,
}

impl ImageAlignment {
    /// Converte o valor persistido no formulário.
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "inline" => Some(Self::Inline),
            "left" => Some(Self::Left),
            "center" => Some(Self::Center),
            "right" => Some(Self::Right),
            _ => None,
        }
    }

    /// Valor estável usado em classe CSS.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Inline => "inline",
            Self::Left => "left",
            Self::Center => "center",
            Self::Right => "right",
        }
    }
}

/// Metadados persistidos de uma inserção de imagem.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InsertedImage {
    /// Fonte relativa ao vault.
    pub src: String,
    /// Alternativa textual.
    pub alt: String,
    /// Título exibido pelo navegador.
    pub title: String,
    /// Legenda visível.
    pub caption: String,
    /// Largura de apresentação em pixels CSS.
    pub width: Option<u32>,
    /// Altura de apresentação em pixels CSS.
    pub height: Option<u32>,
    /// Alinhamento.
    pub alignment: ImageAlignment,
    /// Proporção preservada durante a apresentação.
    pub keep_aspect: bool,
}

impl InsertedImage {
    /// Valida os campos antes de qualquer gravação de asset.
    pub fn validate(&self) -> Result<(), String> {
        if !self.src.starts_with("assets/")
            || self.src.contains("..")
            || self.src.contains(['\n', '\r'])
        {
            return Err("a imagem precisa apontar para um caminho seguro em assets/".into());
        }
        if self.width == Some(0) || self.height == Some(0) {
            return Err("largura e altura precisam ser maiores que zero".into());
        }
        if self.width.unwrap_or(1) > 100_000 || self.height.unwrap_or(1) > 100_000 {
            return Err("largura ou altura fora do limite aceito".into());
        }
        Ok(())
    }

    /// Produz HTML semântico e determinístico, legível dentro do Markdown.
    pub fn to_html(&self) -> Result<String, String> {
        self.validate()?;
        let mut img = format!("<img src=\"{}\" alt=\"{}\"", esc(&self.src), esc(&self.alt));
        if !self.title.is_empty() {
            img.push_str(&format!(" title=\"{}\"", esc(&self.title)));
        }
        if let Some(width) = self.width {
            img.push_str(&format!(" width=\"{width}\""));
        }
        if let Some(height) = self.height {
            img.push_str(&format!(" height=\"{height}\""));
        }
        if self.keep_aspect {
            img.push_str(" class=\"inserted-image__media inserted-image__media--keep-aspect\"");
        } else {
            img.push_str(" class=\"inserted-image__media\"");
        }
        img.push('>');
        Ok(format!(
            "<figure class=\"inserted-image inserted-image--{}\">{}{}</figure>",
            self.alignment.as_str(),
            img,
            if self.caption.is_empty() {
                String::new()
            } else {
                format!("<figcaption>{}</figcaption>", esc(&self.caption))
            },
        ))
    }
}

/// Lê de volta o HTML de [`InsertedImage::to_html`] (ciclo 388) — e um
/// `<img>` solto também. `None` se o bloco não é uma imagem inserida.
pub fn from_html(html: &str) -> Option<InsertedImage> {
    let html = html.trim();
    let e_figura = html.starts_with("<figure") && html.ends_with("</figure>");
    if !e_figura && !(html.starts_with("<img") && html.ends_with('>') && html.matches('<').count() == 1) {
        return None;
    }
    let img = &html[html.find("<img")?..];
    let img = &img[..img.find('>')? + 1];
    let atributo = |nome: &str| -> Option<String> {
        let marca = format!(" {nome}=\"");
        let i = img.find(&marca)? + marca.len();
        let f = img[i..].find('"')?;
        Some(desesc(&img[i..i + f]))
    };
    let classe_da_figura = html.split('>').next().unwrap_or("");
    let alinhamento = ["left", "center", "right"]
        .into_iter()
        .find(|a| classe_da_figura.contains(&format!("inserted-image--{a}")))
        .and_then(ImageAlignment::parse)
        .unwrap_or_default();
    let legenda = html
        .find("<figcaption>")
        .and_then(|i| html[i + 12..].find("</figcaption>").map(|f| desesc(&html[i + 12..i + 12 + f])))
        .unwrap_or_default();
    Some(InsertedImage {
        src: atributo("src")?,
        alt: atributo("alt").unwrap_or_default(),
        title: atributo("title").unwrap_or_default(),
        caption: legenda,
        width: atributo("width").and_then(|v| v.parse().ok()),
        height: atributo("height").and_then(|v| v.parse().ok()),
        alignment: alinhamento,
        keep_aspect: img.contains("inserted-image__media--keep-aspect"),
    })
}

fn desesc(value: &str) -> String {
    value.replace("&quot;", "\"").replace("&lt;", "<").replace("&gt;", ">").replace("&amp;", "&")
}

fn esc(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializa_todos_os_campos_sem_perda() {
        let image = InsertedImage {
            src: "assets/foto-1.png".into(),
            alt: "A & B".into(),
            title: "Título".into(),
            caption: "Legenda <ok>".into(),
            width: Some(640),
            height: Some(480),
            alignment: ImageAlignment::Center,
            keep_aspect: true,
        };
        assert_eq!(image.to_html().unwrap(), "<figure class=\"inserted-image inserted-image--center\"><img src=\"assets/foto-1.png\" alt=\"A &amp; B\" title=\"Título\" width=\"640\" height=\"480\" class=\"inserted-image__media inserted-image__media--keep-aspect\"><figcaption>Legenda &lt;ok&gt;</figcaption></figure>");
    }

    #[test]
    fn recusa_dimensao_zero_e_path_inseguro() {
        let mut image = InsertedImage {
            src: "../x.png".into(),
            ..Default::default()
        };
        assert!(image.validate().is_err());
        image.src = "assets/x.png".into();
        image.width = Some(0);
        assert!(image.validate().is_err());
    }

    #[test]
    fn le_de_volta_o_html_que_escreve() {
        let image = InsertedImage {
            src: "assets/foto-1.png".into(),
            alt: "A & B".into(),
            title: "Título".into(),
            caption: "Legenda <ok>".into(),
            width: Some(640),
            height: None,
            alignment: ImageAlignment::Right,
            keep_aspect: true,
        };
        assert_eq!(from_html(&image.to_html().unwrap()), Some(image));
        assert_eq!(from_html("<img src=\"assets/x.png\" alt=\"x\">").map(|i| i.src), Some("assets/x.png".into()));
        assert_eq!(from_html("<b>não</b>"), None);
    }
}
