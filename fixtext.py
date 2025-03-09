import time
from msvcrt import getch

import pyperclip
import requests

start = time.monotonic()

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


formats = {
    "1": formal,
    "2": cult,
    "3": valle_inclan,
    "4": non_sense,
}

msg = """1. Formal
2. Cult
3. Valle Inclán
4. Non sense\n"""

if __name__ == "__main__":
    print(msg)
    print("Number format to use: ")
    n = getch().decode()
    print(n)
    clipboard_text = pyperclip.paste()
    modified_text = get_modified_text(clipboard_text, formats[n])
    pyperclip.copy(modified_text)
