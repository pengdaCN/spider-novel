#[test]
fn t_match() {
    #[derive(Debug)]
    enum TM {
        T1,
        T2,
        T3,
    }

    fn print_tm(x: TM) {
        match x {
            x @ (TM::T1 | TM::T2) => {
                println!("t1|t2");

                println!("{:?}", x);
            }
            TM::T3 => {
                println!("t3")
            }
        }
    }

    print_tm(TM::T2);
    print_tm(TM::T1);
    print_tm(TM::T3);
}
