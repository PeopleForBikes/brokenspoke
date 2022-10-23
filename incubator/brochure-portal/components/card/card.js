export default function Card(props) {
  return (
    <div className="p-2 lg:w-1/4 md:w-1/2 w-full">
      <div className="flex items-center border-gray-900 border p-4 rounded-lg">
        <img
          alt={props.country}
          className="w-16 h-16 bg-gray-100 object-cover object-center flex-shrink-0 rounded-full mr-4 border"
          src={"https://countryflagsapi.com/svg/" + props.country}
        />
        <div className="flex-grow">
          <h2 className="text-gray-900 title-font font-medium capitalize">
            {props.country}
          </h2>
        </div>
      </div>
    </div>
  );
}
