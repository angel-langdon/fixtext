from pydantic import BaseModel


class Format(BaseModel):
    title: str
    system_instructions: str


_common = """Quiero modifiques el texto final en base a las siguientes pautas:
- Simplifica y corrige errores cuando sea necesario.
- Responde en el mismo lenguaje del texto. En el caso que el texto esté en Español utiliza el dialecto Castellano de España.
- Limítate a modificar el texto, no añadas explicaciones ni comentarios ni comillas.
- Utiliza texto plano, NO añadas carácteres de Markdown, tipo "*", "_", etc. SÍ que puedes añadir "-" para enumerar.
- Mantén anglicismos y palabras de otros idiomas
- Explicalo de la forma más simple posible que sea fácil de entender y esté bien escrito"""

formal = f"""{_common}
- Haz que el mensaje sea formal pero sigue el estilo del mensaje original, no trates de usted si no lo hace el mensaje original"""

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
    Format(title="Fix", system_instructions=_common),
    Format(title="Formal", system_instructions=formal),
    Format(title="Cult", system_instructions=cult),
    Format(title="Valle Inclán", system_instructions=valle_inclan),
    Format(title="Non sense", system_instructions=non_sense),
]
