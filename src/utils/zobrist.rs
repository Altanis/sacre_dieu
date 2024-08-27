use crate::utils::piece::Tile;

use super::{board::Board, consts::PIECE_INDICES, piece::{Piece, PieceColor}};

pub const ZOBRIST_PIECE_KEYS: [[u64; 64]; PIECE_INDICES + 1] = [[12209713866620864367,3699901589224871922,5423274442689130859,17304711928845359584,276586155990948995,4068031863676514951,6443781387327435044,10292535720019985822,16790893840862723372,16322977992133542801,7042105017252067007,12694674137659247527,16737245738399932093,12137319763358584506,12157618083585804299,2189923594599043202,14117278124606628760,508470539879731704,15815027857598713974,16025120356328704504,14063530979081778176,10620554888492879434,10201556053012801997,2908140476134560653,3375014357636861066,9272823946651577125,5008381588612438699,14101338436513503842,13014725010918810067,15872908298655328294,13470523853159375295,12453653561552405154,3129748781895108081,17458472929720554034,4849665670547375400,10581054761878936194,16166703885833379160,15648586837705104382,1567043910540181661,5681641842107339095,1448655543918990749,16306210150073667869,3102762252342971971,5829200677475443302,14106545485987387872,12010948151776098115,2642824478319312610,17291814714332985712,9939347465126682153,1405818557293345203,15041657755219539191,4270855495178832850,9929606297381979947,12392411882859075949,6718232716107015014,9010050912867828908,7073559356140093854,8646073385525222507,151660843512417381,7804484352419027605,13724196636536955255,10723364592271878046,12380151559789214671,14291534850457068945],[17105461470229686444,7803702691817824,9984511824476449005,16144846561141442561,5996737293057502944,1059730393252587734,7879035054659996857,3676019387885327120,10800824768759575012,9463482502405296576,3836192973961154005,9310057257630694736,9009396908598903732,1217946898138935096,510989280182075656,16743430265404223323,17337325502196043192,10175014853447367713,15737120774371653198,16614490280177015225,11784643504727429354,17147774788642330232,12897612088178076717,5867614493915036709,1834650444827252330,1959787030275297017,14347122276521524142,260340770200025230,17458949883446723917,13439286601181772295,2418682021600921762,7080192046742504960,963929494272283327,14152369668267731394,10304385952097843862,466661696482226216,14661648876356063294,14899198312514591129,8658181771791398043,2911277728469185790,18031065809777922768,1300991224923462778,6706952145805931633,347295821102090543,14807294030466566985,16849297743632728942,15853385280929503038,8477166881917052680,18055873370660623998,6148417457975731531,3536016448355136884,6826531177728953360,10230564678587223157,3719847629707555320,1415948547693891400,10734695137078901272,13048363851643081819,6907969902512714562,17311611538654921051,8645768123721094560,16241013196941097550,10848675336811494182,373030212938193285,10529791382464921707],[17368850438167489721,2233006584142601480,1072378029836985416,3250641676354005764,4602618211213879838,18240386060821298633,4728185608297350466,6962905302642989842,16265248837907733410,16617864755319528471,10552300788455783456,6217674826813974823,1936952501924251703,329707590159794274,3751942206548706331,16618606433718056517,9638154184322035182,17847782562036780904,2026821652222315378,16263415158329896991,10820913253278863321,5654343025524585572,4475715776509179682,14206364963331640601,6937917436094846726,6324835114386950481,13486425065802866386,1771267757343095380,14112919118346987785,3140029690715216852,11993195014835413699,1172892673522694893,11722355910285348441,14390432797229158410,4590355836676843448,9974533997602035685,11337793396724475960,711036919368306215,9949663152522016597,2407502707509211410,13655696155255558621,11124354061980226022,11891511824810829119,15230114528303583185,9770990557326698146,15941875245215801247,17888027920172393978,16694314528000142412,7341891992489338152,14440547448399674491,11476845039985635575,15909837159669449829,3711832337624059172,12545717363301936544,2003618867720166071,9808393586516263455,15920534759057494082,12322348428761802350,5059223313995920103,14592401643590002962,1590190207141693971,8354798977781811850,10121048473656734502,3523261724662163261],[7417142651479615489,12205409712341581395,6862692819191711734,14293457161676995540,2418918912714200087,4750592469802318813,13254909231796999427,15711483961591959324,13576478990758489167,8114120700717498061,2679338220976861435,14222747138075441115,18025087134195323982,298552708044257401,8704409624896465398,10123956293391292508,11456344568379618013,4819632741163469577,5254485075593689917,8546308843275275272,17472116284757755229,14373685574430696564,4149474723897798888,5373572652967829495,14035548239335617524,10535762554015655887,15821234833324293926,15498314106823408055,11977827640875902067,5466401983919976652,373617759650931418,15753076121724790777,4143928597117102375,14973439072439443980,4981685052412630593,17336883102744841439,11837077005296488873,10390499843068047053,6097922989154232891,2673608061902125412,1018937525256754925,3531753845211054892,15753790638893024844,14169548004963300526,17957352374455414128,6343575403652706404,17495520440832007636,10752650589254378012,5077553718080046778,11543289546120715177,8258101023883355877,11293963955213395793,17677738261659176514,590573890047108866,1893889501219508745,7290025720779154613,11760513536031638417,13952070569330299906,17548159488896104227,7809809385656615847,7865836344857757445,435108319876164169,10706566736562090180,8889959141964830480],[5020105208603655945,16907844325929097567,5896799335292135781,16174776594573087023,10498793843486938121,3616409800280379028,16227816863843997225,6949775849875351348,1582676162923266320,16940345008574361221,7276556353090582924,17703827687975325203,15229854738689744431,14661190013709072883,9667417347459338320,8709636760075930898,12439620160514294197,7338997561036011931,8416487246193409022,9084670121779892845,10697977178264620656,11473653861625840208,1320938628047405242,18018587647206498844,18189572452673929443,14542971124338812045,7243685307789219907,528412765484590051,3466701495773535277,15231744326344809778,7726976429516324512,9873260937457004813,18003354782996073380,12528166735946912484,1787925114690076919,17418185522938110510,7269848354298160938,16324892890442081044,3920059578205231505,13167123373773391995,5936165854304192785,15221218671175675506,11598293592728307214,13776442903655242195,8610847808355853521,11070024650483108087,5846773567243015564,12540342672246728718,4055782026082886907,17725707195267065017,10630876304569510022,14682898038627460864,3997111186802546735,9417022934412315949,1548884432461216771,8200596104780096013,17338102078461906129,10343937853957004965,11421807017730110594,6378725255061006373,16567593948267594675,15302302662842776261,6529978511708462415,8141421276099380854],[1926882538176693671,15146286426970746146,7533743801507214403,3134649518160387746,8561853661528070890,4052135727054733579,8335178261544286939,12564433943227171040,2132073869374949900,5529167798958749915,6614045599995158666,484496940361946848,13835057796431764881,17880337264172172460,16772843757286682650,6741219189061166657,5863876078751478330,12828878707292120757,1345069471971248999,556310130002752816,2154747741401475317,11668579499995229194,18235555301932537298,6567395156533235128,7394617379527613448,10284821836830428575,3239423531938587766,16332999924027444285,16199600810834172008,12653002218095627206,2999550669167230560,16926826655597780908,4053855522980435129,18083432937549876924,15106815923248198502,8129843256446296924,11205013411108811513,6849476409095747861,16960764990569095500,6331085780444689824,11840122141905913187,17182826785955515886,2309112791058055185,16646828602895954191,2226041340536629181,11928949376259346870,11366970675480793210,14844598272709558969,15700825023064181287,16798141125365838490,15795848776266457744,2535393147095234648,4379420866673217546,266255530236854601,18263965979577267654,12773230544313421965,8716600665140033234,6605194460714017182,9532509725440462596,5748943651029255293,15365530420543703881,10579754519563316402,10490623580863990504,14013110027472873524],[2409213184521995729,4141362321833310041,1659382891306459460,8319285719339782804,3642920663056383524,14078516424016916880,4079914974588476744,13247307529902399156,1200474698369410557,4279596554680060798,5287001119970373620,10890685081284138101,8085232436573133814,71831305870541749,18035021092183672596,7488225739178167711,3467193930028179887,4045960312858615970,16751313437253805617,6707333564102326252,17262782238488699074,5297143202731723452,13143344859896148670,7433784193703398194,420266754065539796,9357786318363323522,15882598021784612578,9678083054168786627,8594941028403885230,8035516381012980330,16320804243842428893,16212930095742438592,870504523573474807,5609206880238649037,7936396823617817251,10526125426687901119,12916553255439063077,8703341496329162501,4140322876117723136,1133460820751884408,13384954575286584777,16396159479655885430,1215493964687098584,13775542534874026493,215476573183513004,55393325750061563,17360099577268921423,10458733347657253473,4159573306080571736,1883404346331870163,13491846137951258207,4774648535400247961,13219328555016437410,4251586608172103166,15292282746629316641,2223564038247191745,6312330251726042588,4013667925712600644,873176310839923771,6449646820625108254,1503801579313505106,1983077391864269706,13420034913511527904,4728301062849851746],[6571395426197756892,15762935183240181782,15214615828124994794,8172135148011996210,10074697782221326812,4119855190485402550,15474649928893467227,17708450986015367488,7296351260675062274,13826483659845104287,9840920556078013369,2736103084218879706,4943363609793964405,11686899504183792216,267152879813695611,953243339317341146,7051936492420023301,3504567926855290207,1773554131373937539,7894673063783668445,10541995390156819537,7416563859545823702,6715757015558279861,3195004780153226269,18036294735999577420,17054264075956610811,16825903592490274897,17292184992821501654,3776738165254175046,12160667009517748326,569109601424985637,8616442438118614999,17538079970284519158,6920755998535279118,2096825845085459488,7544138971888708990,11590068176652617616,14781674658192998285,10080818873617023050,9907336141977613084,15345898708605963369,11631884757933795713,6764160256461040424,5388025564133337146,6884815804065668309,13021457530628463373,14156980112253459009,1042396867964476754,2350853883344940690,3134387080696867480,2897023591121766752,4255060923318991876,12850638589400474368,17140419989882703557,1276231972412826358,7963326971056650652,11425906160609605370,937882059189534560,17231283603654301830,5709688369667379484,6372628658947483190,1778624446572122039,5052294944274596338,716440202442214807],[4544613426697959585,5691803069820048279,14202126735157592788,13974631424628188701,10459453952762781330,12389692858578176322,7945902520264907148,2943820183426421332,16727578385158786850,16857610517542643146,12511929033447372001,8958605654626404778,11699901548855575564,10193585678012291992,7055375539629896501,16224730632636859911,2391728797392800049,8530078636384396804,7700560586347740992,2004228495026678273,270031744615953337,10250459168904606911,6141186156837580096,12920126007578681524,17920063448303375676,4586889809701537518,18227691549609211522,10184206398643682321,18186158517349796774,9948618018105268997,13584863425902534360,4441767863063220375,12173143045152568845,17087352192984068528,13616786981415842531,16199752779043983129,15083450919743241502,1937564822805882047,10564612529294721626,8105031641862390827,9454063773857531466,6860019747254898373,8627093939521800259,10316218072929215523,15638393948473456770,5726938310719545099,910987752290660697,17446223003916869196,7699581398048712533,9408705445686156359,9567176875184746981,1993700577291991221,13236049012694346139,17409634625870337037,3042216899796317594,12777886408462173333,7326898048760858349,6563626327461747657,15093250174514502111,9131153697943212974,5948257873008168679,5732506953466051627,13214197636690921702,3286265856928320586]];
pub const ZOBRIST_CASTLING_KEYS: [u64; 8] = [14038178521941090051,7183301732541184615,4269493297509785763,9513904193794502776,1802736310759650380,10182453658941707750,3930720569150393289,11713858652030083806];
pub const ZOBRIST_EN_PASSANT_KEYS: [u64; 9] = [1624287234879227543,853674592114035455,14230447375709147666,418009804135388707,1158855520242729823,2603517891664097093,11767727142398382975,2245253154515528878,14622179804852566705];
pub const ZOBRIST_SIDE_TO_MOVE: u64 = 9936462436911364648;

/// Generates a zobrist hash given a board state.
pub fn generate_zobrist_hash(board: &Board) -> u64 {
    let mut hash = 0;

    for (i, piece) in board.board.iter().enumerate() {
        if let Some(piece) = piece {
            hash ^= piece.zobrist_key(i);
        }
    }

    hash ^= ZOBRIST_CASTLING_KEYS[board.castle_rights[0] as usize + board.castle_rights[1] as usize];
    
    if let Some(ep) = board.en_passant {
        hash ^= ZOBRIST_EN_PASSANT_KEYS[ep.rank as usize + 1];
    } else {
        hash ^= ZOBRIST_EN_PASSANT_KEYS[0];
    }

    if board.side_to_move == PieceColor::Black {
        hash ^= ZOBRIST_SIDE_TO_MOVE;
    }

    hash
}