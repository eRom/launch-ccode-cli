"""LiteLLM callback: strip 'thinking' content blocks without Anthropic signature.

Claude Code valide une signature cryptographique sur les blocs `thinking`.
Les modèles reasoning hostés ailleurs (Qwen, DeepSeek) renvoient des blocs
thinking sans signature → CC drop la réponse en silence.

Ce callback retire ces blocs problématiques avant que LiteLLM forward la
réponse à `claude`.
"""

from typing import Any

from litellm.integrations.custom_logger import CustomLogger


class StripThinkingCallback(CustomLogger):
    async def async_post_call_success_hook(
        self,
        data: dict[str, Any],
        user_api_key_dict: Any,
        response: Any,
    ) -> Any:
        _strip_thinking_blocks(response)
        return response


def _strip_thinking_blocks(response: Any) -> None:
    """Mute les blocs thinking sans signature dans une réponse Anthropic-shape."""
    content = getattr(response, "content", None)
    if not isinstance(content, list):
        return
    response.content = [b for b in content if not _is_unsigned_thinking(b)]


def _is_unsigned_thinking(block: Any) -> bool:
    if not isinstance(block, dict):
        return False
    return block.get("type") == "thinking" and "signature" not in block


# LiteLLM résout `callbacks: ["lcc_strip_thinking"]` en cherchant cet attribut module-level.
lcc_strip_thinking = StripThinkingCallback()
