import os
from msvcrt import getch

import pyperclip
from dotenv import load_dotenv
from google import genai
from google.genai import types
from pydantic import BaseModel

load_dotenv()


class Format(BaseModel):
    system_instructions: str
    desc: str


GEMINI_API_KEY = os.environ["GEMINI_API_KEY"]

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


def gemini_completion(prompt: str, system_instructions: str):
    client = genai.Client(
        api_key=GEMINI_API_KEY,
    )

    model = "gemini-2.5-flash-preview-05-20"
    contents = [
        types.Content(
            role="user",
            parts=[
                types.Part.from_text(text=prompt),
            ],
        ),
    ]
    generate_content_config = types.GenerateContentConfig(
        response_mime_type="text/plain",
        system_instruction=[
            types.Part.from_text(text=system_instructions),
        ],
        thinking_config=types.ThinkingConfig(thinking_budget=0),
    )

    chunks: list[str] = []
    for chunk in client.models.generate_content_stream(
        model=model,
        contents=contents,
        config=generate_content_config,
    ):
        if chunk.text is None:
            continue
        chunks.append(chunk.text)
        print(chunk.text, end="")
    return "".join(chunks)


formats: list[Format] = [
    Format(system_instructions=formal_english, desc="Formal English"),
    Format(system_instructions=formal, desc="Formal"),
    Format(system_instructions=cult, desc="Cult"),
    Format(system_instructions=valle_inclan, desc="Valle Inclán"),
    Format(system_instructions=non_sense, desc="Non sense"),
]

msg = "\n".join(f"{i}. {f.desc}" for i, f in enumerate(formats)) + "\n"

if __name__ == "__main__":
    print(msg)
    print("Number format to use: ")
    n = getch().decode()
    print(n)
    text = pyperclip.paste()
    i = int(n)
    f = formats[i]
    modified_text = gemini_completion(text, f.system_instructions)
    pyperclip.copy(modified_text)
