from pydantic import BaseModel


class Format(BaseModel):
    title: str
    system_instructions: str


_common = """Quiero modifiques el texto final en base a las siguientes pautas:
- Simplifica y corrige errores cuando sea necesario.
- Responde en el mismo lenguaje del texto. En el caso que el texto esté en Español utiliza el dialecto Castellano de España.
- Limítate a modificar el texto, no añadas explicaciones ni comentarios ni comillas."""

formal = f"""{_common}
- Haz que el mensaje esté bien escrito y sea fácil de entender
- Haz que el mensaje sea formal pero sin pasarte"""

cult = f"""{_common}
- Utiliza un lenguaje antiguo, culto, con léxico elevado y con ciertos insultos complejos y elaborados"""

valle_inclan = f"""{cult}
- Utiliza un lenguaje parecido al de Valle Inclán en Luces de Bohemia y utiliza insultos y palabras en desuso de la época"""

non_sense = f"""{cult}
- Haz un juego de palabras y cambia el significado de la frase para que no tenga ningún sentido lógico pero sí que esté bien escrito"""

formal_english = f"""{formal}
- Answer in English"""

formats: list[Format] = [
    Format(title="Formal English", system_instructions=formal_english),
    Format(title="Formal", system_instructions=formal),
    Format(title="Cult", system_instructions=cult),
    Format(title="Valle Inclán", system_instructions=valle_inclan),
    Format(title="Non sense", system_instructions=non_sense),
]
