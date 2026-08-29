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
}
