# Cas extrêmes de parsing

## Marqueurs consécutifs

****quad bold****

******hex bold******

========double highlight========

+++++++multiple underlines+++++++

## Marqueurs imbriqués pathologiques

***==++Tout mélangé++==***

**Gras avec *italique ==et highlight++** dans gras

==Highlight avec **gras et +++underline== fermeture**

## Images malformées

![Alt text](

![](url_sans_alt)

![alt text](url){malformed attrs w= h= align=}

![](){empty}

## Tables bizarres

| Col1 |
|------|
| Une seule colonne |

|||||||||
|--------|--------|--------|--------|--------|--------|--------|
|||||||||

| **Gras** | *Italique* | ==Highlight== | ++Underline++ |
|----------|------------|---------------|---------------|
| ***Mix*** | ==++Both++== | **Strong ==highlight==** | *Ital ++under++* |

## Lignes très longues

This is an extremely long line that goes on and on and on with lots of **bold** and *italic* and ==highlighted== and ++underlined++ text mixed throughout to test how the parser handles very long input lines that might stress the buffer management system and inline parsing logic when dealing with multiple nested and consecutive formatting markers in a single continuous stream of text content.

## Caractères spéciaux et Unicode

🎉 **Émojis** dans du *formatage* 🚀

Caractères combinés: é̀ à̂ ỗ

Directionnel RTL: مرحبا **عالم** (Arabic)

Symboles: ¡¿ †‡ ‰ ™ © ® ¢ £ ¥ € ₹ ₽