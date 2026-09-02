"""Bounded molecule sources used only by the Ferrum GUI documentation tour."""


# These fixed sources contain no user-controlled path or serialized type choice.
# Ferrum's ordinary CDML/SDF admission remains authoritative (ASVS 1.5.2,
# 2.2.1-2.2.3). Sucrose and distearoylphosphatidylcholine identity derive from
# PubChem CIDs 5988 and 65146; the authored A-T layout supplies only atom
# identity and topology.
# The documentation capture invokes Ferrum's installed RDKit boundary to create
# the phospholipid SDF coordinates and clean, place, and orient the A-T geometry.

SUCROSE_CDML = """<cdml xmlns='urn:ferrum:cdml' version='26.08'>
<molecule id='sucrose' name='Sucrose'>
  <atom id='sucrose-o1' name='O'><point x='460.16' y='500.99'/></atom>
  <atom id='sucrose-o2' name='O'><point x='356.32' y='457.03'/></atom>
  <atom id='sucrose-o3' name='O'><point x='313.20' y='439.17'/></atom>
  <atom id='sucrose-o4' name='O'><point x='442.10' y='402.80'/></atom>
  <atom id='sucrose-o5' name='O'><point x='518.18' y='429.35'/></atom>
  <atom id='sucrose-o6' name='O'><point x='365.56' y='348.47'/></atom>
  <atom id='sucrose-o7' name='O'><point x='294.03' y='329.30'/></atom>
  <atom id='sucrose-o8' name='O'><point x='241.67' y='381.67'/></atom>
  <atom id='sucrose-o9' name='O'><point x='366.24' y='494.06'/></atom>
  <atom id='sucrose-o10' name='O'><point x='502.70' y='545.69'/></atom>
  <atom id='sucrose-o11' name='O'><point x='280.00' y='486.40'/></atom>
  <atom id='sucrose-c1' name='C'><point x='430.37' y='476.87'/></atom>
  <atom id='sucrose-c2' name='C'><point x='444.11' y='441.08'/></atom>
  <atom id='sucrose-c3' name='C'><point x='482.39' y='443.09'/></atom>
  <atom id='sucrose-c4' name='C'><point x='492.31' y='480.12'/></atom>
  <atom id='sucrose-c5' name='C'><point x='346.40' y='420.00'/></atom>
  <atom id='sucrose-c6' name='C'><point x='346.40' y='381.67'/></atom>
  <atom id='sucrose-c7' name='C'><point x='313.20' y='362.50'/></atom>
  <atom id='sucrose-c8' name='C'><point x='280.00' y='381.67'/></atom>
  <atom id='sucrose-c9' name='C'><point x='280.00' y='420.00'/></atom>
  <atom id='sucrose-c10' name='C'><point x='403.26' y='503.98'/></atom>
  <atom id='sucrose-c11' name='C'><point x='516.44' y='509.91'/></atom>
  <atom id='sucrose-c12' name='C'><point x='260.83' y='453.20'/></atom>
  <bond id='sucrose-b1' start='sucrose-o1' end='sucrose-c1' type='n1'/>
  <bond id='sucrose-b2' start='sucrose-o1' end='sucrose-c4' type='n1'/>
  <bond id='sucrose-b3' start='sucrose-o3' end='sucrose-c5' type='n1'/>
  <bond id='sucrose-b4' start='sucrose-o3' end='sucrose-c9' type='n1'/>
  <bond id='sucrose-b5' start='sucrose-o9' end='sucrose-c10' type='n1'/>
  <bond id='sucrose-b6' start='sucrose-o10' end='sucrose-c11' type='n1'/>
  <bond id='sucrose-b7' start='sucrose-o11' end='sucrose-c12' type='n1'/>
  <bond id='sucrose-b8' start='sucrose-c1' end='sucrose-o2' type='n1'/>
  <bond id='sucrose-b9' start='sucrose-c1' end='sucrose-c2' type='n1'/>
  <bond id='sucrose-b10' start='sucrose-c1' end='sucrose-c10' type='h1'/>
  <bond id='sucrose-b11' start='sucrose-c2' end='sucrose-o4' type='w1'/>
  <bond id='sucrose-b12' start='sucrose-c2' end='sucrose-c3' type='n1'/>
  <bond id='sucrose-b13' start='sucrose-c3' end='sucrose-o5' type='h1'/>
  <bond id='sucrose-b14' start='sucrose-c3' end='sucrose-c4' type='n1'/>
  <bond id='sucrose-b15' start='sucrose-c4' end='sucrose-c11' type='w1'/>
  <bond id='sucrose-b16' start='sucrose-c5' end='sucrose-o2' type='w1'/>
  <bond id='sucrose-b17' start='sucrose-c5' end='sucrose-c6' type='n1'/>
  <bond id='sucrose-b18' start='sucrose-c6' end='sucrose-o6' type='w1'/>
  <bond id='sucrose-b19' start='sucrose-c6' end='sucrose-c7' type='n1'/>
  <bond id='sucrose-b20' start='sucrose-c7' end='sucrose-o7' type='h1'/>
  <bond id='sucrose-b21' start='sucrose-c7' end='sucrose-c8' type='n1'/>
  <bond id='sucrose-b22' start='sucrose-c8' end='sucrose-o8' type='w1'/>
  <bond id='sucrose-b23' start='sucrose-c8' end='sucrose-c9' type='n1'/>
  <bond id='sucrose-b24' start='sucrose-c9' end='sucrose-c12' type='h1'/>
</molecule>
</cdml>"""


DISTEAROYLPHOSPHATIDYLCHOLINE_SMILES = (
	"CCCCCCCCCCCCCCCCCC(=O)OCC(COP(=O)([O-])OCC[N+](C)(C)C)"
	"OC(=O)CCCCCCCCCCCCCCCCC"
)


DNA_BASE_PAIR_CDML = """<cdml xmlns='urn:ferrum:cdml' version='26.08'>
<text id='base-pair-label'>
  <point x='370' y='205'/><font size='14' color='#374151'/>
  <ftext>Watson-Crick A-T base pair</ftext>
</text>
<molecule id='thymine' name='Thymine'>
  <atom id='t-n1' name='N'><point x='220' y='330'/></atom>
  <atom id='t-c2' name='C'><point x='260' y='310'/></atom>
  <atom id='t-o2' name='O'><point x='275' y='270'/></atom>
  <atom id='t-n3' name='N'><point x='300' y='330'/></atom>
  <atom id='t-h3' name='H'><point x='335' y='330'/></atom>
  <atom id='t-c4' name='C'><point x='300' y='370'/></atom>
  <atom id='t-o4' name='O'><point x='340' y='370'/></atom>
  <atom id='t-c5' name='C'><point x='260' y='390'/></atom>
  <atom id='t-c7' name='C'><point x='260' y='435'/></atom>
  <atom id='t-c6' name='C'><point x='220' y='370'/></atom>
  <bond id='t-b1' start='t-n1' end='t-c2' type='n1'/>
  <bond id='t-b2' start='t-c2' end='t-o2' type='n2'/>
  <bond id='t-b3' start='t-c2' end='t-n3' type='n1'/>
  <bond id='t-b4' start='t-n3' end='t-h3' type='n1'/>
  <bond id='t-b5' start='t-n3' end='t-c4' type='n1'/>
  <bond id='t-b6' start='t-c4' end='t-o4' type='n2'/>
  <bond id='t-b7' start='t-c4' end='t-c5' type='n1'/>
  <bond id='t-b8' start='t-c5' end='t-c7' type='n1'/>
  <bond id='t-b9' start='t-c5' end='t-c6' type='n2'/>
  <bond id='t-b10' start='t-c6' end='t-n1' type='n1'/>
</molecule>
<molecule id='adenine' name='Adenine'>
  <atom id='a-n1' name='N'><point x='460' y='330'/></atom>
  <atom id='a-c2' name='C'><point x='495' y='310'/></atom>
  <atom id='a-n3' name='N'><point x='530' y='330'/></atom>
  <atom id='a-c4' name='C'><point x='530' y='370'/></atom>
  <atom id='a-c5' name='C'><point x='495' y='390'/></atom>
  <atom id='a-c6' name='C'><point x='460' y='370'/></atom>
  <atom id='a-n6' name='N'><point x='420' y='390'/></atom>
  <atom id='a-h6a' name='H'><point x='390' y='380'/></atom>
  <atom id='a-h6b' name='H'><point x='390' y='410'/></atom>
  <atom id='a-n7' name='N'><point x='565' y='385'/></atom>
  <atom id='a-c8' name='C'><point x='580' y='425'/></atom>
  <atom id='a-n9' name='N'><point x='525' y='445'/></atom>
  <bond id='a-b1' start='a-n1' end='a-c2' type='n2'/>
  <bond id='a-b2' start='a-c2' end='a-n3' type='n1'/>
  <bond id='a-b3' start='a-n3' end='a-c4' type='n2'/>
  <bond id='a-b4' start='a-c4' end='a-c5' type='n1'/>
  <bond id='a-b5' start='a-c5' end='a-c6' type='n2'/>
  <bond id='a-b6' start='a-c6' end='a-n1' type='n1'/>
  <bond id='a-b7' start='a-c6' end='a-n6' type='n1'/>
  <bond id='a-b8' start='a-n6' end='a-h6a' type='n1'/>
  <bond id='a-b9' start='a-n6' end='a-h6b' type='n1'/>
  <bond id='a-b10' start='a-c4' end='a-n7' type='n1'/>
  <bond id='a-b11' start='a-n7' end='a-c8' type='n2'/>
  <bond id='a-b12' start='a-c8' end='a-n9' type='n1'/>
  <bond id='a-b13' start='a-n9' end='a-c5' type='n1'/>
</molecule>
</cdml>"""
