#[derive(Clone, PartialEq, Debug)]
pub enum Exercise {
    PronunciationAndMeaning(char),
    Handwriting(char),
    // A placeholder for the next exercise type to be developed.
    Placeholder,
}