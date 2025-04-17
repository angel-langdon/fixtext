from msvcrt import getch
from typing import TypedDict

import pyperclip
import requests

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

formal_english = """I want you to modify the final text based on the following guidelines:
- Simplify and correct errors when necessary.
- Limit yourself to modifying the text, do not add explanations, comments or quotation marks.
- Make the message well written and easy to understand
- Make the message formal but do not overdo it."""


def get_modified_text(text: str, system_content: str) -> str:
    body = {
        "messages": [
            {"role": "system", "content": system_content},
            {"role": "user", "content": text},
        ],
        "model": "openai-large",
        "seed": 42,
        "jsonMode": False,
        "private": True,
    }
    r = requests.post("https://text.pollinations.ai/", json=body)
    return r.content.decode()


class Format(TypedDict):
    system_content: str
    desc: str


formats: list[Format] = [
    {"system_content": formal_english, "desc": "Formal English"},
    {"system_content": formal, "desc": "Formal"},
    {"system_content": cult, "desc": "Cult"},
    {"system_content": valle_inclan, "desc": "Valle Inclán"},
    {"system_content": non_sense, "desc": "Non sense"},
]

msg = "\n".join(f"{i}. {dic['desc']}" for i, dic in enumerate(formats)) + "\n"

if __name__ == "__main__":
    print(msg)
    print("Number format to use: ")
    n = getch().decode()
    print(n)
    text = pyperclip.paste()
    i = int(n)
    system_content = formats[i]["system_content"]
    modified_text = get_modified_text(text, system_content)
    pyperclip.copy(modified_text)
