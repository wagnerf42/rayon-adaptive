pub trait Divisible: Sized {
    fn should_be_divided(&self) -> bool;
    /// Divide Self into two parts.
    /// It's better if the two parts contain roughly an equivalent amount of work.
    /// For Indexed iterators we REQUIRE an object of size n to be cut into two objects of size
    /// floor(n/2), ceil(n/2).
    fn divide(self) -> (Self, Self);
    /// If we can, divide at given index.
    fn divide_at(self, index: usize) -> (Self, Self) {
        self.divide()
    }
}
